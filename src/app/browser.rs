use super::settings::BrowserSettings;
use super::tab::Tab;
use crate::layout::tree::ScrollbarHitArea;
use crate::net::{AsyncLoader, LoadProgress, Loader};
use crate::render::gpu::GpuContext;
use crate::render::painter::{Color, DisplayCommand, Painter, Rect};
use crate::render::text::TextRenderer;
use crate::ui::Chrome;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use wgpu::*;

pub struct Browser {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub loader: Loader,
    pub async_loader: AsyncLoader,
    pub chrome: Chrome,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub chrome_height: f32,
    pub settings: BrowserSettings,
    pub runtime: Runtime,
    progress_rx: Option<mpsc::UnboundedReceiver<LoadProgress>>,
    /// Cached display commands to avoid rebuilding every frame
    cached_rects: Vec<(Rect, Color)>,
    cached_texts: Vec<(String, f32, f32, Color, f32)>,
    cached_images: Vec<(Rect, usize, f32)>,
    /// Whether the cache needs to be rebuilt
    display_list_dirty: bool,
}

impl Browser {
    pub fn new(viewport_width: f32, viewport_height: f32, settings: BrowserSettings) -> Self {
        let chrome_height = 50.0;
        let content_height = viewport_height - chrome_height;

        let tab = Tab::new(0, viewport_width, content_height, settings.clone());

        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        Self {
            tabs: vec![tab],
            active_tab: 0,
            loader: Loader::new(),
            async_loader: runtime.block_on(async { AsyncLoader::new() }),
            chrome: Chrome::new(viewport_width, chrome_height, &settings),
            viewport_width,
            viewport_height,
            chrome_height,
            settings,
            runtime,
            progress_rx: None,
            cached_rects: Vec::new(),
            cached_texts: Vec::new(),
            cached_images: Vec::new(),
            display_list_dirty: true,
        }
    }

    /// Mark display list as needing rebuild (call after content changes)
    pub fn invalidate_display_list(&mut self) {
        self.display_list_dirty = true;
    }

    pub fn load_html_to_active_tab(&mut self, html: &str, text_renderer: &mut TextRenderer) {
        let tab = &mut self.tabs[self.active_tab];
        tab.load_html(html, &self.loader, text_renderer);
        self.display_list_dirty = true;
    }

    pub fn toggle_js(&mut self, text_renderer: &mut TextRenderer) {
        self.settings.js_enabled = !self.settings.js_enabled;
        for tab in &mut self.tabs {
            tab.settings.js_enabled = self.settings.js_enabled;
        }
        self.chrome.update_toggle_state(&self.settings);
        self.reload_current_page(text_renderer);
    }

    pub fn toggle_css(&mut self, text_renderer: &mut TextRenderer) {
        self.settings.css_enabled = !self.settings.css_enabled;
        for tab in &mut self.tabs {
            tab.settings.css_enabled = self.settings.css_enabled;
        }
        self.chrome.update_toggle_state(&self.settings);
        self.reload_current_page(text_renderer);
    }

    fn reload_current_page(&mut self, text_renderer: &mut TextRenderer) {
        let tab = &self.tabs[self.active_tab];
        if let Some(url) = tab.url.clone() {
            let url_str = url.to_string();
            self.navigate(&url_str, text_renderer);
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn navigate(&mut self, url: &str, _text_renderer: &mut TextRenderer) {
        self.navigate_async(url);
    }

    /// Start async navigation
    pub fn navigate_async(&mut self, url: &str) {
        let tab = &mut self.tabs[self.active_tab];

        match tab.start_async_load(url) {
            Ok(cancel_token) => {
                // Update chrome URL
                if let Some(parsed_url) = &tab.url {
                    self.chrome.set_url(&parsed_url.to_string());
                }

                // Start loading via async loader
                let rx = self.runtime.block_on(async {
                    self.async_loader.load(tab.url.clone().unwrap(), cancel_token)
                });

                self.progress_rx = Some(rx);
                self.chrome.set_loading(true);
            }
            Err(e) => {
                tab.fail_load(e);
            }
        }
    }

    /// Poll for loading progress updates
    pub fn poll_loading(&mut self, text_renderer: &mut TextRenderer) {
        if let Some(rx) = &mut self.progress_rx {
            // Only process one message per frame to allow progress bar to render
            if let Ok(progress) = rx.try_recv() {
                match progress {
                    LoadProgress::Started { content_length, .. } => {
                        log::info!("Loading started, content_length: {:?}", content_length);
                        let tab = &mut self.tabs[self.active_tab];
                        if let Some(lp) = &mut tab.loading_progress {
                            lp.total_bytes = content_length;
                        }
                    }
                    LoadProgress::Progress { bytes_received, total_bytes } => {
                        log::info!("Loading progress: {} / {:?}", bytes_received, total_bytes);
                        let tab = &mut self.tabs[self.active_tab];
                        tab.update_progress(bytes_received, total_bytes);

                        // Update chrome progress
                        if let Some(lp) = &tab.loading_progress {
                            self.chrome.set_progress(Some(lp.fraction()));
                            log::info!("Progress bar fraction: {}", lp.fraction());
                        }
                        self.display_list_dirty = true;
                    }
                    LoadProgress::Complete { body, final_url } => {
                        log::info!("Loading complete, body length: {}, final_url: {}", body.len(), final_url);
                        let tab = &mut self.tabs[self.active_tab];
                        // Update URL to final URL after redirects
                        tab.url = Some(final_url.clone());
                        self.chrome.set_url(&final_url.to_string());
                        tab.complete_load(&body, text_renderer);
                        // Collect pending images for loading
                        tab.collect_pending_images();
                        self.chrome.set_loading(false);
                        self.chrome.set_progress(None);
                        self.progress_rx = None;
                        self.display_list_dirty = true;
                        return;
                    }
                    LoadProgress::Error { message } => {
                        let tab = &mut self.tabs[self.active_tab];
                        tab.fail_load(message);
                        self.chrome.set_loading(false);
                        self.chrome.set_progress(None);
                        self.progress_rx = None;
                        return;
                    }
                    LoadProgress::Cancelled => {
                        let tab = &mut self.tabs[self.active_tab];
                        tab.loading = false;
                        tab.loading_progress = None;
                        self.chrome.set_loading(false);
                        self.chrome.set_progress(None);
                        self.progress_rx = None;
                        return;
                    }
                }
            }
        }
    }

    /// Cancel any in-progress loading
    pub fn cancel_loading(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        tab.cancel_load();
        self.chrome.set_loading(false);
        self.chrome.set_progress(None);
        self.progress_rx = None;
    }

    /// Stop loading (called from chrome stop button)
    pub fn stop(&mut self) {
        self.cancel_loading();
    }

    /// Reload the current page
    pub fn reload(&mut self, text_renderer: &mut TextRenderer) {
        let url = self.tabs[self.active_tab].url.as_ref().map(|u| u.to_string());
        if let Some(url) = url {
            self.navigate(&url, text_renderer);
        }
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.tabs[self.active_tab].loading
    }

    /// Update render statistics for display in chrome
    /// Note: Does NOT invalidate display list - FPS is updated in chrome but
    /// we always rebuild chrome display list (it's cheap) while caching content
    pub fn set_render_stats(&mut self, fps: f32, render_time_ms: f32) {
        self.chrome.set_render_stats(fps, render_time_ms);
    }

    /// Check if there are pending images to load
    pub fn has_pending_images(&self) -> bool {
        self.tabs[self.active_tab].has_pending_images()
    }

    /// Load pending images using the GPU context
    pub fn load_pending_images(&mut self, gpu: &GpuContext, text_renderer: &mut TextRenderer) {
        let tab = &mut self.tabs[self.active_tab];
        tab.load_images_sync(&self.loader, gpu, text_renderer);
        self.display_list_dirty = true;
    }

    /// Load pending images without re-layout (faster for screenshot mode)
    pub fn load_pending_images_fast(&mut self, gpu: &GpuContext) {
        let tab = &mut self.tabs[self.active_tab];
        tab.load_images_sync_fast(&self.loader, gpu);
        self.display_list_dirty = true;
    }

    /// Re-collect pending images considering only those in the visible viewport
    /// This is useful for screenshot mode where we only need visible images
    pub fn collect_visible_images(&mut self) {
        let viewport_height = self.viewport_height - self.chrome_height;
        let tab = &mut self.tabs[self.active_tab];
        tab.collect_pending_images_in_viewport(Some(viewport_height));
    }

    pub fn go_back(&mut self, _text_renderer: &mut TextRenderer) {
        let tab = &mut self.tabs[self.active_tab];
        if let Some(url) = tab.history.go_back().map(|s| s.to_string()) {
            self.navigate_async(&url);
        }
    }

    pub fn go_forward(&mut self, _text_renderer: &mut TextRenderer) {
        let tab = &mut self.tabs[self.active_tab];
        if let Some(url) = tab.history.go_forward().map(|s| s.to_string()) {
            self.navigate_async(&url);
        }
    }

    pub fn handle_click(&mut self, x: i32, y: i32, text_renderer: &mut TextRenderer) {
        let x = x as f32;
        let y = y as f32;

        if y < self.chrome_height {
            if let Some(action) = self.chrome.handle_click(x, y) {
                match action {
                    crate::ui::ChromeAction::Back => self.go_back(text_renderer),
                    crate::ui::ChromeAction::Forward => self.go_forward(text_renderer),
                    crate::ui::ChromeAction::Navigate(url) => self.navigate(&url, text_renderer),
                    crate::ui::ChromeAction::FocusUrlBar => {}
                    crate::ui::ChromeAction::ToggleJs => self.toggle_js(text_renderer),
                    crate::ui::ChromeAction::ToggleCss => self.toggle_css(text_renderer),
                    crate::ui::ChromeAction::Stop => self.stop(),
                    crate::ui::ChromeAction::Reload => self.reload(text_renderer),
                }
            }
        } else {
            let content_y = y - self.chrome_height;

            // Check scrollbar hit first
            let scrollbar_hit = self.active_tab().layout_tree.scrollbar_hit_test(x, content_y);
            match scrollbar_hit {
                ScrollbarHitArea::Thumb => {
                    self.active_tab_mut().layout_tree.begin_thumb_drag(content_y);
                    return;
                }
                ScrollbarHitArea::Track => {
                    let scroll_pos = self.active_tab().layout_tree.track_y_to_scroll(content_y);
                    self.active_tab_mut().scroll_to(scroll_pos);
                    self.display_list_dirty = true;
                    return;
                }
                ScrollbarHitArea::None => {}
            }

            let tab = &self.tabs[self.active_tab];
            if let Some(node_id) = tab.hit_test(x, content_y) {
                if let Some(href) = tab.get_link_at(node_id) {
                    let full_url = if href.starts_with("http") {
                        href
                    } else if let Some(base) = &tab.url {
                        base.join(&href).map(|u| u.to_string()).unwrap_or(href)
                    } else {
                        href
                    };
                    self.navigate(&full_url, text_renderer);
                }
            }
        }
    }

    pub fn handle_scroll(&mut self, delta: i32) {
        let scroll_amount = delta as f32 * 40.0;
        self.active_tab_mut().scroll_smooth(-scroll_amount);
        self.display_list_dirty = true;
    }

    pub fn handle_mouse_move(&mut self, x: i32, y: i32) {
        let x = x as f32;
        let y = y as f32;

        if y < self.chrome_height {
            return;
        }

        let content_y = y - self.chrome_height;

        if self.active_tab().layout_tree.is_dragging_scrollbar() {
            self.active_tab_mut().layout_tree.update_thumb_drag(content_y);
            self.display_list_dirty = true;
            return;
        }

        self.active_tab_mut().layout_tree.update_scrollbar_hover(x, content_y);
    }

    pub fn handle_mouse_up(&mut self) {
        self.active_tab_mut().layout_tree.end_thumb_drag();
    }

    pub fn update_scroll(&mut self, dt: f32) -> bool {
        let animating = self.active_tab_mut().update_scroll(dt);
        if animating {
            self.display_list_dirty = true;
        }
        animating
    }

    pub fn is_animating(&self) -> bool {
        self.active_tab().is_scroll_animating() || self.is_loading()
    }

    pub fn handle_text_input(&mut self, text: &str) {
        self.chrome.handle_text_input(text);
        self.display_list_dirty = true;
    }

    pub fn handle_key(&mut self, keycode: sdl2::keyboard::Keycode, text_renderer: &mut TextRenderer) {
        if let Some(action) = self.chrome.handle_key(keycode) {
            match action {
                crate::ui::ChromeAction::Navigate(url) => self.navigate(&url, text_renderer),
                _ => {}
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, text_renderer: &mut TextRenderer) {
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;

        let content_height = self.viewport_height - self.chrome_height;
        self.chrome.resize(self.viewport_width);

        for tab in &mut self.tabs {
            tab.resize(self.viewport_width, content_height, text_renderer);
        }
        self.display_list_dirty = true;
    }

    pub fn render(
        &mut self,
        gpu: &GpuContext,
        painter: &Painter,
        text_renderer: &mut TextRenderer,
        scale_factor: f32,
    ) {
        let output = match gpu.get_current_texture() {
            Ok(o) => o,
            Err(SurfaceError::Lost) => return,
            Err(SurfaceError::OutOfMemory) => panic!("Out of GPU memory"),
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Clear pass
        {
            let _clear_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Use same logic as render_to_view for consistency
        self.render_to_view(
            gpu,
            painter,
            text_renderer,
            &mut encoder,
            &view,
            self.viewport_width,
            self.viewport_height,
            scale_factor,
            true, // include chrome
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        text_renderer.trim();
    }

    /// Render to a specific texture view (used for screenshots and offscreen rendering).
    /// Uses the exact same drawing logic as render().
    pub fn render_to_view(
        &mut self,
        gpu: &GpuContext,
        painter: &Painter,
        text_renderer: &mut TextRenderer,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
        include_chrome: bool,
    ) {
        // Build content display list (no caching for screenshot - one-shot render)
        let mut content_rects = Vec::new();
        let mut content_texts = Vec::new();
        let mut content_images = Vec::new();

        let y_offset = if include_chrome { self.chrome_height } else { 0.0 };
        let content_list = self.active_tab().build_display_list();

        Self::collect_display_commands(
            &content_list.commands,
            y_offset,
            &mut content_rects,
            &mut content_texts,
            &mut content_images,
        );

        // Chrome display list (if included)
        let mut chrome_rects = Vec::new();
        let mut chrome_texts = Vec::new();
        if include_chrome {
            let chrome_list = self.chrome.build_display_list();
            Self::collect_display_commands(&chrome_list.commands, 0.0, &mut chrome_rects, &mut chrome_texts, &mut Vec::new());
        }

        let chrome_height = if include_chrome { self.chrome_height } else { 0.0 };

        // Use shared drawing logic
        Self::draw_frame(
            gpu,
            painter,
            text_renderer,
            encoder,
            view,
            &content_rects,
            &content_texts,
            &content_images,
            &chrome_rects,
            &chrome_texts,
            &self.active_tab().image_cache,
            viewport_width,
            viewport_height,
            scale_factor,
            chrome_height,
        );
    }

    /// Core drawing logic shared by render() and render_to_view().
    /// This ensures identical rendering output regardless of the render target.
    fn draw_frame(
        gpu: &GpuContext,
        painter: &Painter,
        text_renderer: &mut TextRenderer,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        content_rects: &[(Rect, Color)],
        content_texts: &[(String, f32, f32, Color, f32)],
        content_images: &[(Rect, usize, f32)],
        chrome_rects: &[(Rect, Color)],
        chrome_texts: &[(String, f32, f32, Color, f32)],
        image_cache: &crate::render::ImageCache,
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
        chrome_height: f32,
    ) {
        // Combine all rects into a single draw call to avoid buffer synchronization issues
        // Content rects first (behind), then chrome rects on top
        let mut all_rects: Vec<(Rect, Color)> = Vec::with_capacity(content_rects.len() + chrome_rects.len());
        all_rects.extend_from_slice(content_rects);
        all_rects.extend_from_slice(chrome_rects);

        if !all_rects.is_empty() {
            painter.draw_rects(
                gpu,
                encoder,
                view,
                &all_rects,
                viewport_width,
                viewport_height,
                scale_factor,
            );
        }

        // Render images with viewport culling
        for (rect, texture_id, opacity) in content_images {
            // Skip images outside viewport
            if rect.y + rect.height < chrome_height || rect.y > viewport_height {
                continue;
            }

            if let Some(texture_view) = image_cache.get_texture_view(*texture_id) {
                painter.draw_image(
                    gpu,
                    encoder,
                    view,
                    texture_view,
                    rect,
                    *opacity,
                    viewport_width,
                    viewport_height,
                    scale_factor,
                );
            }
        }

        // Render all text in a single pass
        let content_clip_top = (chrome_height * scale_factor) as u32;
        let physical_width = (viewport_width * scale_factor) as u32;
        let physical_height = (viewport_height * scale_factor) as u32;

        let text_groups: Vec<(&[(String, f32, f32, Color, f32)], u32)> = if !chrome_texts.is_empty() {
            vec![
                (content_texts, content_clip_top),
                (chrome_texts, 0),
            ]
        } else {
            vec![(content_texts, content_clip_top)]
        };

        text_renderer.render_all(
            gpu,
            encoder,
            view,
            &text_groups,
            physical_width,
            physical_height,
        );
    }

    fn collect_display_commands(
        commands: &[DisplayCommand],
        y_offset: f32,
        rects: &mut Vec<(Rect, Color)>,
        texts: &mut Vec<(String, f32, f32, Color, f32)>,
        images: &mut Vec<(Rect, usize, f32)>,
    ) {
        for cmd in commands {
            match cmd {
                DisplayCommand::SolidRect { rect, color, opacity, .. } => {
                    // Apply opacity to the color
                    let mut c = *color;
                    c.a *= opacity;
                    rects.push((
                        Rect::new(rect.x, rect.y + y_offset, rect.width, rect.height),
                        c,
                    ));
                }
                DisplayCommand::Text { text, x, y, color, font_size, opacity } => {
                    let mut c = *color;
                    c.a *= opacity;
                    texts.push((text.clone(), *x, *y + y_offset, c, *font_size));
                }
                DisplayCommand::Border { rect, color, width, .. } => {
                    let r = Rect::new(rect.x, rect.y + y_offset, rect.width, rect.height);
                    rects.push((Rect::new(r.x, r.y, r.width, *width), *color));
                    rects.push((Rect::new(r.x, r.bottom() - *width, r.width, *width), *color));
                    rects.push((Rect::new(r.x, r.y, *width, r.height), *color));
                    rects.push((Rect::new(r.right() - *width, r.y, *width, r.height), *color));
                }
                DisplayCommand::Image { rect, texture_id, opacity } => {
                    images.push((
                        Rect::new(rect.x, rect.y + y_offset, rect.width, rect.height),
                        *texture_id,
                        *opacity,
                    ));
                }
                DisplayCommand::BoxShadow { rect, color, offset_x, offset_y, blur_radius, spread_radius, .. } => {
                    // Render shadow as a slightly larger rect behind the element
                    let shadow_rect = Rect::new(
                        rect.x + offset_x - spread_radius,
                        rect.y + y_offset + offset_y - spread_radius,
                        rect.width + spread_radius * 2.0,
                        rect.height + spread_radius * 2.0,
                    );
                    // Apply blur by reducing opacity based on blur radius
                    let mut c = *color;
                    if *blur_radius > 0.0 {
                        c.a *= 0.5; // Simplified blur effect
                    }
                    rects.push((shadow_rect, c));
                }
            }
        }
    }
}
