use crate::render::text::TextRenderer;

pub struct TextLayout {
    pub lines: Vec<TextLine>,
    pub total_width: f32,
    pub total_height: f32,
}

pub struct TextLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TextLayout {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            total_width: 0.0,
            total_height: 0.0,
        }
    }

    pub fn layout_text(
        text: &str,
        max_width: f32,
        font_size: f32,
        line_height: f32,
        text_renderer: &mut TextRenderer,
    ) -> Self {
        let mut layout = TextLayout::new();
        let mut current_y = 0.0;

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        let mut line_width = 0.0;

        let space_width = text_renderer.measure_text(" ", font_size).0;

        for word in words {
            let (word_width, _) = text_renderer.measure_text(word, font_size);

            if line_width + word_width > max_width && !current_line.is_empty() {
                let height = font_size * line_height;
                layout.lines.push(TextLine {
                    text: current_line.clone(),
                    x: 0.0,
                    y: current_y,
                    width: line_width,
                    height,
                });
                layout.total_width = layout.total_width.max(line_width);
                current_y += height;

                current_line = word.to_string();
                line_width = word_width;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    line_width += space_width;
                }
                current_line.push_str(word);
                line_width += word_width;
            }
        }

        if !current_line.is_empty() {
            let height = font_size * line_height;
            layout.lines.push(TextLine {
                text: current_line,
                x: 0.0,
                y: current_y,
                width: line_width,
                height,
            });
            layout.total_width = layout.total_width.max(line_width);
            current_y += height;
        }

        layout.total_height = current_y;
        layout
    }
}

impl Default for TextLayout {
    fn default() -> Self {
        Self::new()
    }
}
