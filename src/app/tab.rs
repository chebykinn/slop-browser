use super::history::History;
use super::settings::BrowserSettings;
use crate::css::{parse_css, StyleComputer, Stylesheet};
use crate::dom::{parse_html, Document, NodeId};
use crate::js::{Interpreter, Lexer, Parser};
use crate::layout::LayoutTree;
use crate::net::{CancelToken, Loader};
use crate::render::painter::{Color, DisplayList, Rect};
use crate::render::text::TextRenderer;
use crate::render::{ImageCache, ImageSize, decode_image, decode_data_url};
use crate::render::gpu::GpuContext;
use std::rc::Rc;
use std::time::Instant;
use url::Url;

/// Loading progress information
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub bytes_received: u64,
    pub total_bytes: Option<u64>,
    pub url: String,
}

impl LoadingProgress {
    pub fn new(url: String) -> Self {
        Self {
            bytes_received: 0,
            total_bytes: None,
            url,
        }
    }

    /// Returns progress as a fraction 0.0-1.0
    /// If total is unknown, returns 0.3 (30%) to show indeterminate progress
    pub fn fraction(&self) -> f32 {
        match self.total_bytes {
            Some(total) if total > 0 => (self.bytes_received as f32 / total as f32).min(1.0),
            _ => 0.3, // Indeterminate progress
        }
    }
}

pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: Option<Url>,
    pub document: Document,
    pub stylesheets: Vec<Rc<Stylesheet>>,
    pub style_computer: StyleComputer,
    pub layout_tree: LayoutTree,
    pub history: History,
    pub loading: bool,
    pub error: Option<String>,
    pub settings: BrowserSettings,
    pub loading_progress: Option<LoadingProgress>,
    pub cancel_token: Option<CancelToken>,
    pub image_cache: ImageCache,
    /// URLs of images that need to be loaded
    pending_images: Vec<String>,
}

impl Tab {
    pub fn new(id: usize, viewport_width: f32, viewport_height: f32, settings: BrowserSettings) -> Self {
        Self {
            id,
            title: String::from("New Tab"),
            url: None,
            document: Document::new(),
            stylesheets: Vec::new(),
            style_computer: StyleComputer::new(viewport_width, viewport_height),
            layout_tree: LayoutTree::new(viewport_width, viewport_height),
            history: History::new(),
            loading: false,
            error: None,
            settings,
            loading_progress: None,
            cancel_token: None,
            image_cache: ImageCache::new(),
            pending_images: Vec::new(),
        }
    }

    /// Start an async load - returns the cancel token
    pub fn start_async_load(&mut self, url: &str) -> Result<CancelToken, String> {
        // Cancel any existing load
        self.cancel_load();

        self.loading = true;
        self.error = None;

        let url_str = if !url.contains("://") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        let parsed_url = Url::parse(&url_str).map_err(|e| format!("Invalid URL: {}", e))?;

        self.url = Some(parsed_url);
        self.loading_progress = Some(LoadingProgress::new(url_str));

        let cancel_token = CancelToken::new();
        self.cancel_token = Some(cancel_token.clone());

        Ok(cancel_token)
    }

    /// Cancel any in-progress load
    pub fn cancel_load(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
        self.loading = false;
        self.loading_progress = None;
    }

    /// Update progress from async loader
    pub fn update_progress(&mut self, bytes_received: u64, total_bytes: Option<u64>) {
        if let Some(progress) = &mut self.loading_progress {
            progress.bytes_received = bytes_received;
            progress.total_bytes = total_bytes;
        }
    }

    /// Mark loading as complete (for async loading - without external resources)
    pub fn complete_load(&mut self, html: &str, text_renderer: &mut TextRenderer) {
        self.loading = false;
        self.loading_progress = None;
        self.cancel_token = None;

        if let Some(url) = &self.url {
            self.history.push(url.to_string());
            self.title = url.host_str().unwrap_or("Unknown").to_string();
        }

        // For async loading, we don't have a loader for external resources
        // Use load_html_simple which doesn't load external stylesheets
        self.load_html_simple(html, text_renderer);
    }

    /// Mark loading as failed
    pub fn fail_load(&mut self, error: String) {
        self.loading = false;
        self.loading_progress = None;
        self.cancel_token = None;
        self.error = Some(error);
    }

    pub fn load_url(&mut self, url_str: &str, loader: &Loader, text_renderer: &mut TextRenderer) {
        self.loading = true;
        self.error = None;

        let url_str = if !url_str.contains("://") {
            format!("https://{}", url_str)
        } else {
            url_str.to_string()
        };

        let url = match Url::parse(&url_str) {
            Ok(u) => u,
            Err(e) => {
                self.error = Some(format!("Invalid URL: {}", e));
                self.loading = false;
                return;
            }
        };

        match loader.fetch(&url) {
            Ok(html) => {
                self.url = Some(url.clone());
                self.history.push(url.to_string());
                self.title = url.host_str().unwrap_or("Unknown").to_string();

                self.load_html(&html, loader, text_renderer);
            }
            Err(e) => {
                self.error = Some(format!("Failed to load: {}", e));
            }
        }

        self.loading = false;
    }

    /// Load HTML with full external resource support (requires loader)
    pub fn load_html(&mut self, html: &str, loader: &Loader, text_renderer: &mut TextRenderer) {
        let total_start = Instant::now();

        let parse_start = Instant::now();
        self.document = parse_html(html);
        let parse_time = parse_start.elapsed();

        self.stylesheets.clear();

        let default_css = include_str!("../../assets/default.css");
        self.stylesheets.push(Rc::new(parse_css(default_css)));

        if self.settings.css_enabled {
            self.extract_styles();
            self.load_external_stylesheets(loader);
        }

        self.style_computer.clear_stylesheets();
        for stylesheet in &self.stylesheets {
            self.style_computer.add_stylesheet(Rc::clone(stylesheet));
        }

        let style_start = Instant::now();
        self.style_computer.compute_styles(&self.document);
        let style_time = style_start.elapsed();

        let layout_start = Instant::now();
        self.layout_tree.build(&self.document, &self.style_computer, text_renderer);
        let layout_time = layout_start.elapsed();

        // Collect pending images for loading
        self.collect_pending_images();

        if self.settings.js_enabled {
            self.execute_scripts();
        }

        let total_time = total_start.elapsed();
        println!(
            "[Page render] total={:.2}ms (parse={:.2}ms, style={:.2}ms, layout={:.2}ms) nodes={}",
            total_time.as_secs_f32() * 1000.0,
            parse_time.as_secs_f32() * 1000.0,
            style_time.as_secs_f32() * 1000.0,
            layout_time.as_secs_f32() * 1000.0,
            self.document.node_count()
        );
    }

    /// Load HTML without external resource loading (for async loaded content)
    fn load_html_simple(&mut self, html: &str, text_renderer: &mut TextRenderer) {
        let total_start = Instant::now();

        let parse_start = Instant::now();
        self.document = parse_html(html);
        let parse_time = parse_start.elapsed();

        self.stylesheets.clear();

        let default_css = include_str!("../../assets/default.css");
        self.stylesheets.push(Rc::new(parse_css(default_css)));

        if self.settings.css_enabled {
            self.extract_styles();
            // Skip external stylesheets for now in async mode
        }

        self.style_computer.clear_stylesheets();
        for stylesheet in &self.stylesheets {
            self.style_computer.add_stylesheet(Rc::clone(stylesheet));
        }

        let style_start = Instant::now();
        self.style_computer.compute_styles(&self.document);
        let style_time = style_start.elapsed();

        let layout_start = Instant::now();
        self.layout_tree.build(&self.document, &self.style_computer, text_renderer);
        let layout_time = layout_start.elapsed();

        if self.settings.js_enabled {
            self.execute_scripts();
        }

        let total_time = total_start.elapsed();
        println!(
            "[Page render] total={:.2}ms (parse={:.2}ms, style={:.2}ms, layout={:.2}ms) nodes={}",
            total_time.as_secs_f32() * 1000.0,
            parse_time.as_secs_f32() * 1000.0,
            style_time.as_secs_f32() * 1000.0,
            layout_time.as_secs_f32() * 1000.0,
            self.document.node_count()
        );
    }

    fn extract_styles(&mut self) {
        let style_elements = self.document.get_elements_by_tag_name("style");
        for node_id in style_elements {
            let css_text = self.document.get_text_content(node_id);
            if !css_text.is_empty() {
                self.stylesheets.push(Rc::new(parse_css(&css_text)));
            }
        }
    }

    fn load_external_stylesheets(&mut self, loader: &Loader) {
        let link_elements = self.document.get_elements_by_tag_name("link");
        for node_id in link_elements {
            if let Some(node) = self.document.get_node(node_id) {
                if let Some(elem) = node.as_element() {
                    let rel = elem.get_attribute("rel");
                    let href = elem.get_attribute("href");

                    if rel == Some("stylesheet") {
                        if let Some(href) = href {
                            // Resolve relative URL against base
                            let css_url = if let Some(base) = &self.url {
                                match base.join(href) {
                                    Ok(u) => Some(u),
                                    Err(_) => None,
                                }
                            } else if href.starts_with("http") {
                                Url::parse(href).ok()
                            } else {
                                None
                            };

                            if let Some(url) = css_url {
                                if let Ok(css_text) = loader.fetch(&url) {
                                    self.stylesheets.push(Rc::new(parse_css(&css_text)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn execute_scripts(&mut self) {
        let script_elements = self.document.get_elements_by_tag_name("script");
        let mut interpreter = Interpreter::new();

        for node_id in script_elements {
            // Skip external scripts (src attribute) for now
            if let Some(node) = self.document.get_node(node_id) {
                if let Some(elem) = node.as_element() {
                    // Skip scripts with src attribute
                    if elem.get_attribute("src").is_some() {
                        continue;
                    }
                }
            }

            let script_content = self.document.get_text_content(node_id);
            if !script_content.trim().is_empty() {
                let mut lexer = Lexer::new(&script_content);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(tokens);
                let statements = parser.parse();
                interpreter.execute(&statements);
            }
        }
    }

    pub fn resize(&mut self, width: f32, height: f32, text_renderer: &mut TextRenderer) {
        let start = Instant::now();

        self.style_computer.set_viewport(width, height);
        self.layout_tree.viewport_width = width;
        self.layout_tree.viewport_height = height;

        // Skip style recomputation on resize - styles are viewport-independent
        // (vh/vw units are applied during layout, not during style computation)
        // This significantly improves resize performance for large documents
        self.layout_tree.build(&self.document, &self.style_computer, text_renderer);

        log::debug!(
            "Resize to {}x{}: {:.2}ms (nodes={})",
            width,
            height,
            start.elapsed().as_secs_f32() * 1000.0,
            self.document.node_count()
        );
    }

    pub fn scroll(&mut self, delta: f32) {
        self.layout_tree.scroll(delta);
    }

    pub fn scroll_smooth(&mut self, delta: f32) {
        self.layout_tree.scroll_smooth(delta);
    }

    pub fn update_scroll(&mut self, dt: f32) -> bool {
        self.layout_tree.update_scroll(dt)
    }

    pub fn scroll_to(&mut self, position: f32) {
        self.layout_tree.scroll_to(position);
    }

    pub fn scroll_immediate(&mut self, position: f32) {
        self.layout_tree.scroll_immediate(position);
    }

    pub fn is_scroll_animating(&self) -> bool {
        self.layout_tree.is_scroll_animating()
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        self.layout_tree.hit_test(x, y)
    }

    pub fn get_link_at(&self, node_id: NodeId) -> Option<String> {
        let mut current = Some(node_id);

        while let Some(id) = current {
            if let Some(node) = self.document.get_node(id) {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name == "a" {
                        return elem.get_attribute("href").map(|s| s.to_string());
                    }
                }
                current = node.parent;
            } else {
                break;
            }
        }

        None
    }

    pub fn go_back(&mut self, loader: &Loader, text_renderer: &mut TextRenderer) -> bool {
        if let Some(url) = self.history.go_back().map(|s| s.to_string()) {
            if let Ok(parsed) = Url::parse(&url) {
                if let Ok(html) = loader.fetch(&parsed) {
                    self.url = Some(parsed);
                    self.load_html(&html, loader, text_renderer);
                    return true;
                }
            }
        }
        false
    }

    pub fn go_forward(&mut self, loader: &Loader, text_renderer: &mut TextRenderer) -> bool {
        if let Some(url) = self.history.go_forward().map(|s| s.to_string()) {
            if let Ok(parsed) = Url::parse(&url) {
                if let Ok(html) = loader.fetch(&parsed) {
                    self.url = Some(parsed);
                    self.load_html(&html, loader, text_renderer);
                    return true;
                }
            }
        }
        false
    }

    pub fn build_display_list(&self) -> DisplayList {
        // Show error page if there's an error
        if let Some(error) = &self.error {
            return self.build_error_display_list(error);
        }

        let mut list = self.layout_tree.build_display_list(self.layout_tree.scroll_y);
        self.layout_tree.render_scrollbar(&mut list);
        list
    }

    fn build_error_display_list(&self, error: &str) -> DisplayList {
        let mut list = DisplayList::new();

        // Background
        list.push_rect(
            Rect::new(0.0, 0.0, self.layout_tree.viewport_width, self.layout_tree.viewport_height),
            Color::rgb(250, 240, 240),
        );

        // Error icon area
        let center_x = self.layout_tree.viewport_width / 2.0;

        // Title
        list.push_text(
            "Page Load Error".to_string(),
            center_x - 80.0,
            100.0,
            Color::rgb(180, 60, 60),
            24.0,
        );

        // Error message
        let error_lines: Vec<&str> = error.lines().collect();
        let mut y = 150.0;
        for line in error_lines {
            // Wrap long lines
            let max_width = 60; // characters
            let chunks: Vec<&str> = line
                .as_bytes()
                .chunks(max_width)
                .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
                .collect();

            for chunk in chunks {
                list.push_text(
                    chunk.to_string(),
                    50.0,
                    y,
                    Color::rgb(80, 80, 80),
                    14.0,
                );
                y += 20.0;
            }
        }

        // URL if available
        if let Some(url) = &self.url {
            y += 20.0;
            list.push_text(
                format!("URL: {}", url),
                50.0,
                y,
                Color::rgb(100, 100, 100),
                12.0,
            );
        }

        list
    }

    /// Collect image URLs from the layout tree that need to be loaded
    pub fn collect_pending_images(&mut self) {
        // Resolve URLs in the layout tree and get list of all image URLs
        let urls = self.layout_tree.resolve_image_urls(self.url.as_ref());
        self.pending_images.clear();

        log::info!("Found {} image URLs in layout tree", urls.len());
        for url in &urls {
            log::info!("  Image URL: {}", url);
        }

        for url in urls {
            // Skip if already cached
            if !self.image_cache.contains(&url) {
                self.pending_images.push(url);
            }
        }
        log::info!("Pending images to load: {}", self.pending_images.len());
    }

    /// Load pending images synchronously using the loader
    pub fn load_images_sync(&mut self, loader: &Loader, gpu: &GpuContext, text_renderer: &mut TextRenderer) {
        let pending = std::mem::take(&mut self.pending_images);
        let loaded_count = pending.len();

        for url in pending {
            self.image_cache.start_loading(&url);

            let result = if url.starts_with("data:") {
                // Handle data URLs
                decode_data_url(&url)
            } else {
                // Fetch from network
                match Url::parse(&url) {
                    Ok(parsed_url) => {
                        match loader.fetch_bytes(&parsed_url) {
                            Ok(bytes) => decode_image(&bytes),
                            Err(e) => Err(format!("Failed to fetch image: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Invalid URL: {}", e)),
                }
            };

            match result {
                Ok(image) => {
                    log::info!("Loaded image {}: {}x{}", url, image.width(), image.height());
                    let size = ImageSize::new(image.width(), image.height());
                    self.image_cache.store_image(&url, &image, gpu);

                    // Update layout tree with texture ID
                    if let Some(texture_id) = self.image_cache.get_texture_id(&url) {
                        log::info!("Stored texture {} for {}", texture_id, url);
                        self.layout_tree.update_image_texture(&url, texture_id, size);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to load image {}: {}", url, e);
                    self.image_cache.store_error(&url, e);
                }
            }
        }

        // Re-layout if we loaded any images (sizes may have changed)
        if loaded_count > 0 {
            self.layout_tree.build(&self.document, &self.style_computer, text_renderer);
            // Re-resolve URLs to update texture IDs after rebuild
            self.layout_tree.resolve_image_urls(self.url.as_ref());
            // Re-apply texture IDs from cache
            for url in self.image_cache.urls() {
                if let Some(texture_id) = self.image_cache.get_texture_id(url) {
                    if let Some(size) = self.image_cache.get_size(url) {
                        self.layout_tree.update_image_texture(url, texture_id, size);
                    }
                }
            }
        }
    }

    /// Check if there are pending images to load
    pub fn has_pending_images(&self) -> bool {
        !self.pending_images.is_empty()
    }

    /// Get the number of pending images
    pub fn pending_image_count(&self) -> usize {
        self.pending_images.len()
    }
}
