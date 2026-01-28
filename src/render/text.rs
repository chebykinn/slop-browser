use super::gpu::GpuContext;
use super::painter::Color;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use glyphon::{
    Cache, FontSystem as GlyphonFontSystem, SwashCache as GlyphonSwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};
use std::collections::HashMap;
use wgpu::*;

/// Key for caching shaped text buffers
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct TextCacheKey {
    text: String,
    font_size_bits: u32, // f32 as bits for hashing
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    glyphon_font_system: GlyphonFontSystem,
    glyphon_swash_cache: GlyphonSwashCache,
    cache: Cache,
    text_atlas: TextAtlas,
    text_renderer: GlyphonRenderer,
    viewport: Viewport,
    /// Cache of shaped text buffers keyed by (text, font_size)
    buffer_cache: HashMap<TextCacheKey, usize>,
    /// Pool of buffers (indices into this vec are stored in buffer_cache)
    buffers: Vec<glyphon::Buffer>,
    /// Cache of text measurements keyed by (text, font_size) -> (width, height)
    measure_cache: HashMap<TextCacheKey, (f32, f32)>,
    /// Character width lookup table (ASCII 0-127) at reference font size 16.0
    char_widths: [f32; 128],
    /// Average width for non-ASCII characters at reference font size
    default_char_width: f32,
    scale_factor: f32,
}

impl TextRenderer {
    pub fn new(gpu: &GpuContext, scale_factor: f32) -> Self {
        use std::time::Instant;
        let t0 = Instant::now();

        let mut font_system = FontSystem::new();
        let t1 = Instant::now();

        let swash_cache = SwashCache::new();

        // Pre-compute character widths for fast measurement (reuse font_system)
        let (char_widths, default_char_width) = Self::compute_char_widths(&mut font_system);
        let t2 = Instant::now();

        let glyphon_font_system = GlyphonFontSystem::new();
        let t3 = Instant::now();

        let glyphon_swash_cache = GlyphonSwashCache::new();

        println!(
            "[TextRenderer init] font_system={:.0}ms char_widths={:.0}ms glyphon_font={:.0}ms",
            (t1 - t0).as_secs_f32() * 1000.0,
            (t2 - t1).as_secs_f32() * 1000.0,
            (t3 - t2).as_secs_f32() * 1000.0,
        );
        let cache = Cache::new(&gpu.device);
        let mut text_atlas = TextAtlas::new(&gpu.device, &gpu.queue, &cache, gpu.format());
        let text_renderer = GlyphonRenderer::new(
            &mut text_atlas,
            &gpu.device,
            MultisampleState::default(),
            None,
        );

        let viewport = Viewport::new(&gpu.device, &cache);

        Self {
            font_system,
            swash_cache,
            glyphon_font_system,
            glyphon_swash_cache,
            cache,
            text_atlas,
            text_renderer,
            viewport,
            buffer_cache: HashMap::new(),
            buffers: Vec::new(),
            measure_cache: HashMap::new(),
            char_widths,
            default_char_width,
            scale_factor,
        }
    }

    /// Pre-compute character widths at reference font size (16.0)
    fn compute_char_widths(font_system: &mut FontSystem) -> ([f32; 128], f32) {
        const REF_SIZE: f32 = 16.0;
        let metrics = Metrics::new(REF_SIZE, REF_SIZE * 1.2);
        let mut widths = [0.0f32; 128];

        // Measure each ASCII character
        for i in 32u8..127u8 {
            let ch = i as char;
            let text = ch.to_string();
            let mut buffer = Buffer::new(font_system, metrics);
            let attrs = Attrs::new().family(Family::SansSerif);
            buffer.set_text(font_system, &text, attrs, Shaping::Basic);
            buffer.shape_until_scroll(font_system, false);

            let mut width = 0.0f32;
            for run in buffer.layout_runs() {
                width = width.max(run.line_w);
            }
            widths[i as usize] = width;
        }

        // Space and common control chars
        widths[32] = widths['m' as usize] * 0.3; // space ~30% of 'm'
        widths[9] = widths[' ' as usize] * 4.0;  // tab = 4 spaces

        // Default width for non-ASCII (use 'M' width as approximation for wide chars)
        let default_width = widths['M' as usize];

        (widths, default_width)
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        if (self.scale_factor - scale_factor).abs() > 0.001 {
            // Scale factor changed - invalidate all cached buffers
            self.buffer_cache.clear();
            self.measure_cache.clear();
        }
        self.scale_factor = scale_factor;
    }

    /// Clear the text shaping cache (call when content changes significantly)
    pub fn clear_cache(&mut self) {
        self.buffer_cache.clear();
        self.measure_cache.clear();
    }

    /// Render all text groups in a single pass
    /// Each group has its own clip_top value (in physical pixels)
    /// Groups are rendered in order: first group's texts, then second group's texts, etc.
    pub fn render_all(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        text_groups: &[(&[(String, f32, f32, Color, f32)], u32)], // (texts, clip_top in physical pixels)
        viewport_width: u32,
        viewport_height: u32,
    ) {
        // Count total texts
        let total_texts: usize = text_groups.iter().map(|(texts, _)| texts.len()).sum();
        if total_texts == 0 {
            return;
        }

        self.viewport.update(
            &gpu.queue,
            glyphon::Resolution {
                width: viewport_width,
                height: viewport_height,
            },
        );

        // Collect all buffer indices, positions, colors, and clip bounds
        struct TextData {
            buffer_idx: usize,
            x: f32,
            y: f32,
            color: Color,
            clip_top: u32,
        }
        let mut all_text_data: Vec<TextData> = Vec::with_capacity(total_texts);

        for (texts, clip_top) in text_groups.iter() {
            for (text, x, y, color, font_size) in texts.iter() {
                let physical_font_size = *font_size * self.scale_factor;
                let cache_key = TextCacheKey {
                    text: text.clone(),
                    font_size_bits: physical_font_size.to_bits(),
                };

                let buffer_idx = if let Some(&idx) = self.buffer_cache.get(&cache_key) {
                    idx
                } else {
                    let new_idx = self.buffers.len();
                    self.buffers.push(glyphon::Buffer::new(
                        &mut self.glyphon_font_system,
                        glyphon::Metrics::new(16.0, 20.0),
                    ));

                    let buffer = &mut self.buffers[new_idx];
                    buffer.set_metrics(
                        &mut self.glyphon_font_system,
                        glyphon::Metrics::new(physical_font_size, physical_font_size * 1.2),
                    );
                    let attrs = glyphon::Attrs::new().family(glyphon::Family::SansSerif);
                    buffer.set_text(
                        &mut self.glyphon_font_system,
                        text,
                        attrs,
                        glyphon::Shaping::Advanced,
                    );
                    buffer.set_size(
                        &mut self.glyphon_font_system,
                        Some(viewport_width as f32),
                        None,
                    );
                    buffer.shape_until_scroll(&mut self.glyphon_font_system, false);

                    self.buffer_cache.insert(cache_key, new_idx);
                    new_idx
                };

                all_text_data.push(TextData {
                    buffer_idx,
                    x: *x,
                    y: *y,
                    color: *color,
                    clip_top: *clip_top,
                });
            }
        }

        // Build text areas using collected data
        let text_areas: Vec<TextArea> = all_text_data
            .iter()
            .map(|data| TextArea {
                buffer: &self.buffers[data.buffer_idx],
                left: data.x * self.scale_factor,
                top: data.y * self.scale_factor,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: data.clip_top as i32,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgba(
                    (data.color.r * 255.0) as u8,
                    (data.color.g * 255.0) as u8,
                    (data.color.b * 255.0) as u8,
                    (data.color.a * 255.0) as u8,
                ),
                custom_glyphs: &[],
            })
            .collect();

        self.text_renderer
            .prepare(
                &gpu.device,
                &gpu.queue,
                &mut self.glyphon_font_system,
                &mut self.text_atlas,
                &self.viewport,
                text_areas,
                &mut self.glyphon_swash_cache,
            )
            .unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer
                .render(&self.text_atlas, &self.viewport, &mut render_pass)
                .unwrap();
        }
    }

    pub fn trim(&mut self) {
        self.text_atlas.trim();
    }

    /// Fast text measurement using pre-computed character width table
    /// O(n) where n is string length - no buffer creation or shaping
    /// Accounts for text wrapping at max_width
    pub fn measure_text_fast(&self, text: &str, font_size: f32, max_width: f32) -> (f32, f32) {
        const REF_SIZE: f32 = 16.0;
        let scale = font_size / REF_SIZE;
        let line_height = font_size * 1.2;

        let mut max_line_width = 0.0f32;
        let mut current_line_width = 0.0f32;
        let mut lines = 1;

        // Track word for wrapping
        let mut word_width = 0.0f32;

        for ch in text.chars() {
            if ch == '\n' {
                // Explicit newline
                current_line_width += word_width;
                max_line_width = max_line_width.max(current_line_width);
                current_line_width = 0.0;
                word_width = 0.0;
                lines += 1;
            } else if ch == ' ' || ch == '\t' {
                // Space - commit word and add space
                current_line_width += word_width;
                word_width = 0.0;

                let space_width = if ch == '\t' {
                    self.char_widths[' ' as usize] * scale * 4.0
                } else {
                    self.char_widths[' ' as usize] * scale
                };

                // Check if we need to wrap before adding space
                if current_line_width + space_width > max_width && current_line_width > 0.0 {
                    max_line_width = max_line_width.max(current_line_width);
                    current_line_width = 0.0;
                    lines += 1;
                } else {
                    current_line_width += space_width;
                }
            } else {
                // Regular character - add to current word
                let char_width = if (ch as u32) < 128 {
                    self.char_widths[ch as usize]
                } else {
                    self.default_char_width
                };
                word_width += char_width * scale;

                // Check if word + current line exceeds max width
                if current_line_width + word_width > max_width && current_line_width > 0.0 {
                    // Wrap: start new line with current word
                    max_line_width = max_line_width.max(current_line_width);
                    current_line_width = 0.0;
                    lines += 1;
                }
            }
        }

        // Don't forget the last word
        current_line_width += word_width;
        max_line_width = max_line_width.max(current_line_width);

        (max_line_width.min(max_width), line_height * lines as f32)
    }

    pub fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        // Check cache first
        let cache_key = TextCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
        };

        if let Some(&dims) = self.measure_cache.get(&cache_key) {
            return dims;
        }

        // Cache miss - measure the text with full Advanced shaping
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let attrs = Attrs::new().family(Family::SansSerif);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += metrics.line_height;
        }

        let result = (width, height.max(metrics.line_height));

        // Cache the result
        self.measure_cache.insert(cache_key, result);

        result
    }
}

/// Standalone text measurement without GPU context (for testing and layout)
pub struct TextMeasurer {
    font_system: FontSystem,
}

impl TextMeasurer {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
        }
    }

    pub fn measure(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let attrs = Attrs::new().family(Family::SansSerif);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += metrics.line_height;
        }

        (width, height.max(metrics.line_height))
    }

    pub fn measure_with_max_width(&mut self, text: &str, font_size: f32, max_width: f32) -> (f32, f32) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(&mut self.font_system, Some(max_width), None);
        let attrs = Attrs::new().family(Family::SansSerif);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += metrics.line_height;
        }

        (width.min(max_width), height.max(metrics.line_height))
    }
}

impl Default for TextMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TextMeasurer Tests ====================

    #[test]
    fn test_measure_empty_string() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("", 16.0);

        // Empty string should have zero width but still have line height
        assert_eq!(width, 0.0);
        assert!(height > 0.0, "Height should be at least line height");
    }

    #[test]
    fn test_measure_single_character() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("A", 16.0);

        assert!(width > 0.0, "Single character should have width");
        assert!(height > 0.0, "Single character should have height");
    }

    #[test]
    fn test_measure_simple_text() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("Hello", 16.0);

        assert!(width > 0.0, "Text should have width");
        assert!(height > 0.0, "Text should have height");
    }

    #[test]
    fn test_measure_longer_text_is_wider() {
        let mut measurer = TextMeasurer::new();
        let (short_width, _) = measurer.measure("Hi", 16.0);
        let (long_width, _) = measurer.measure("Hello World", 16.0);

        assert!(long_width > short_width, "Longer text should be wider");
    }

    #[test]
    fn test_measure_larger_font_is_bigger() {
        let mut measurer = TextMeasurer::new();
        let (small_width, small_height) = measurer.measure("Test", 12.0);
        let (large_width, large_height) = measurer.measure("Test", 24.0);

        assert!(large_width > small_width, "Larger font should be wider");
        assert!(large_height > small_height, "Larger font should be taller");
    }

    #[test]
    fn test_measure_multiline_text() {
        let mut measurer = TextMeasurer::new();
        let (_, single_height) = measurer.measure("Line", 16.0);
        let (_, multi_height) = measurer.measure("Line\nLine\nLine", 16.0);

        // Three lines should be approximately 3x the height of one line
        assert!(multi_height > single_height * 2.0, "Multiline should be taller");
    }

    #[test]
    fn test_measure_with_max_width_wrapping() {
        let mut measurer = TextMeasurer::new();
        let long_text = "This is a very long line of text that should wrap";

        let (_unwrapped_width, unwrapped_height) = measurer.measure(long_text, 16.0);
        let (wrapped_width, wrapped_height) = measurer.measure_with_max_width(long_text, 16.0, 100.0);

        assert!(wrapped_width <= 100.0, "Wrapped width should be within max");
        assert!(wrapped_height >= unwrapped_height, "Wrapped text should be at least as tall");
    }

    #[test]
    fn test_measure_spaces() {
        let mut measurer = TextMeasurer::new();
        let (no_space_width, _) = measurer.measure("AB", 16.0);
        let (with_space_width, _) = measurer.measure("A B", 16.0);

        assert!(with_space_width > no_space_width, "Space should add width");
    }

    #[test]
    fn test_measure_special_characters() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("@#$%^&*()", 16.0);

        assert!(width > 0.0, "Special characters should have width");
        assert!(height > 0.0, "Special characters should have height");
    }

    #[test]
    fn test_measure_unicode() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("日本語", 16.0);

        assert!(width > 0.0, "Unicode should have width");
        assert!(height > 0.0, "Unicode should have height");
    }

    #[test]
    fn test_measure_emoji() {
        let mut measurer = TextMeasurer::new();
        let (_width, height) = measurer.measure("🎉🚀", 16.0);

        // Emoji rendering may vary, but should have some dimensions
        assert!(height > 0.0, "Emoji should have height");
    }

    #[test]
    fn test_measure_consistent_results() {
        let mut measurer = TextMeasurer::new();
        let text = "Consistent";

        let (width1, height1) = measurer.measure(text, 16.0);
        let (width2, height2) = measurer.measure(text, 16.0);

        assert_eq!(width1, width2, "Same text should have same width");
        assert_eq!(height1, height2, "Same text should have same height");
    }

    #[test]
    fn test_measure_different_font_sizes() {
        let mut measurer = TextMeasurer::new();
        let text = "Size";

        let sizes = [8.0, 12.0, 16.0, 24.0, 32.0, 48.0];
        let mut prev_width = 0.0;
        let mut prev_height = 0.0;

        for size in sizes {
            let (width, height) = measurer.measure(text, size);
            assert!(width > prev_width, "Width should increase with font size");
            assert!(height > prev_height, "Height should increase with font size");
            prev_width = width;
            prev_height = height;
        }
    }

    #[test]
    fn test_measure_tabs() {
        let mut measurer = TextMeasurer::new();
        let (no_tab_width, _) = measurer.measure("AB", 16.0);
        let (with_tab_width, _) = measurer.measure("A\tB", 16.0);

        // Tab should add some width (exact amount varies by renderer)
        assert!(with_tab_width >= no_tab_width, "Tab should not reduce width");
    }

    #[test]
    fn test_measure_very_small_font_size() {
        let mut measurer = TextMeasurer::new();
        // Note: cosmic-text doesn't allow zero font size, so test with very small
        let (width, height) = measurer.measure("Test", 1.0);

        // Very small font size should still have positive dimensions
        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn test_measure_very_large_font() {
        let mut measurer = TextMeasurer::new();
        let (width, height) = measurer.measure("Big", 200.0);

        assert!(width > 100.0, "Large font should have significant width");
        assert!(height > 100.0, "Large font should have significant height");
    }

    #[test]
    fn test_measure_with_max_width_no_wrap_needed() {
        let mut measurer = TextMeasurer::new();
        let short_text = "Hi";

        let (normal_width, normal_height) = measurer.measure(short_text, 16.0);
        let (constrained_width, constrained_height) = measurer.measure_with_max_width(short_text, 16.0, 1000.0);

        // If max_width is larger than text, dimensions should be similar
        assert!((normal_width - constrained_width).abs() < 1.0);
        assert!((normal_height - constrained_height).abs() < 1.0);
    }

    #[test]
    fn test_measurer_default() {
        let measurer = TextMeasurer::default();
        assert!(std::mem::size_of_val(&measurer) > 0);
    }

    // ==================== Color Tests ====================

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE.r, 1.0);
        assert_eq!(Color::WHITE.g, 1.0);
        assert_eq!(Color::WHITE.b, 1.0);
        assert_eq!(Color::WHITE.a, 1.0);

        assert_eq!(Color::BLACK.r, 0.0);
        assert_eq!(Color::BLACK.g, 0.0);
        assert_eq!(Color::BLACK.b, 0.0);
        assert_eq!(Color::BLACK.a, 1.0);

        assert_eq!(Color::TRANSPARENT.a, 0.0);
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color.r, 1.0);
        assert!((color.g - 0.502).abs() < 0.01);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_rgba() {
        let color = Color::rgba(255, 255, 255, 128);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
        assert!((color.a - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_color_to_array() {
        let color = Color::rgb(255, 0, 128);
        let arr = color.to_array();
        assert_eq!(arr[0], 1.0);
        assert_eq!(arr[1], 0.0);
        assert!((arr[2] - 0.502).abs() < 0.01);
        assert_eq!(arr[3], 1.0);
    }

    #[test]
    fn test_color_from_hex_6_digit() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_from_hex_3_digit() {
        let color = Color::from_hex("#F00").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_from_hex_8_digit() {
        let color = Color::from_hex("#FF000080").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert!((color.a - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hex_no_hash() {
        let color = Color::from_hex("00FF00").unwrap();
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_from_hex_invalid() {
        assert!(Color::from_hex("invalid").is_none());
        assert!(Color::from_hex("#GG0000").is_none());
        assert!(Color::from_hex("#12345").is_none());
    }

    // ==================== TextCacheKey Tests ====================

    #[test]
    fn test_cache_key_equality_same_text_same_size() {
        let key1 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_inequality_different_text() {
        let key1 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "World".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_inequality_different_size() {
        let key1 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 24.0f32.to_bits(),
        };
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let key1 = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 14.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 14.0f32.to_bits(),
        };

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_cache_key_hash_different_for_different_keys() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let key1 = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 14.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "Different".to_string(),
            font_size_bits: 14.0f32.to_bits(),
        };

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);

        // Different keys should (almost always) have different hashes
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_cache_key_in_hashmap() {
        let mut cache: HashMap<TextCacheKey, usize> = HashMap::new();

        let key1 = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        cache.insert(key1.clone(), 0);

        // Same key should retrieve the same value
        let lookup_key = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_eq!(cache.get(&lookup_key), Some(&0));

        // Different text should not find
        let different_text = TextCacheKey {
            text: "World".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_eq!(cache.get(&different_text), None);

        // Different size should not find
        let different_size = TextCacheKey {
            text: "Hello".to_string(),
            font_size_bits: 24.0f32.to_bits(),
        };
        assert_eq!(cache.get(&different_size), None);
    }

    #[test]
    fn test_cache_key_multiple_entries() {
        let mut cache: HashMap<TextCacheKey, usize> = HashMap::new();

        // Insert multiple unique keys
        let entries: Vec<(&str, f32, usize)> = vec![
            ("Hello", 16.0, 0),
            ("Hello", 24.0, 1),  // Same text, different size
            ("World", 16.0, 2),  // Different text, same size
            ("Test", 12.0, 3),
            ("FPS: 60", 10.0, 4),  // Example FPS counter text
            ("https://example.com", 14.0, 5),  // Example URL text
        ];

        for (text, size, idx) in &entries {
            let key = TextCacheKey {
                text: text.to_string(),
                font_size_bits: (*size).to_bits(),
            };
            cache.insert(key, *idx);
        }

        assert_eq!(cache.len(), 6);

        // Verify all entries are retrievable
        for (text, size, idx) in &entries {
            let key = TextCacheKey {
                text: text.to_string(),
                font_size_bits: (*size).to_bits(),
            };
            assert_eq!(cache.get(&key), Some(idx));
        }
    }

    #[test]
    fn test_cache_key_empty_text() {
        let key1 = TextCacheKey {
            text: String::new(),
            font_size_bits: 16.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: String::new(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_eq!(key1, key2);

        // Empty text is different from non-empty
        let key3 = TextCacheKey {
            text: " ".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_key_special_characters() {
        let mut cache: HashMap<TextCacheKey, usize> = HashMap::new();

        let special_texts = vec![
            "日本語",
            "🎉🚀",
            "<>&\"'",
            "line1\nline2",
            "\t\t",
            "60.0 FPS | 16.7ms",
        ];

        for (idx, text) in special_texts.iter().enumerate() {
            let key = TextCacheKey {
                text: text.to_string(),
                font_size_bits: 14.0f32.to_bits(),
            };
            cache.insert(key, idx);
        }

        assert_eq!(cache.len(), special_texts.len());

        // All should be retrievable
        for (idx, text) in special_texts.iter().enumerate() {
            let key = TextCacheKey {
                text: text.to_string(),
                font_size_bits: 14.0f32.to_bits(),
            };
            assert_eq!(cache.get(&key), Some(&idx));
        }
    }

    #[test]
    fn test_cache_key_font_size_precision() {
        // Test that slightly different font sizes create different keys
        let key1 = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };
        let key2 = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 16.001f32.to_bits(),
        };

        // These should be different because the bits are different
        assert_ne!(key1.font_size_bits, key2.font_size_bits);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_clone() {
        let key1 = TextCacheKey {
            text: "Clone Test".to_string(),
            font_size_bits: 18.0f32.to_bits(),
        };
        let key2 = key1.clone();

        assert_eq!(key1, key2);
        assert_eq!(key1.text, key2.text);
        assert_eq!(key1.font_size_bits, key2.font_size_bits);
    }

    #[test]
    fn test_cache_key_overwrite() {
        let mut cache: HashMap<TextCacheKey, usize> = HashMap::new();

        let key = TextCacheKey {
            text: "Test".to_string(),
            font_size_bits: 16.0f32.to_bits(),
        };

        cache.insert(key.clone(), 0);
        assert_eq!(cache.get(&key), Some(&0));

        // Overwrite with same key
        cache.insert(key.clone(), 99);
        assert_eq!(cache.get(&key), Some(&99));
        assert_eq!(cache.len(), 1);  // Still just one entry
    }
}
