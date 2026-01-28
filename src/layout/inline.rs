use super::tree::{BoxType, LayoutBox};
use crate::render::text::TextRenderer;

pub struct InlineFormattingContext {
    pub line_boxes: Vec<LineBox>,
    pub current_x: f32,
    pub current_y: f32,
    pub line_height: f32,
    pub max_width: f32,
}

pub struct LineBox {
    pub fragments: Vec<InlineFragment>,
    pub y: f32,
    pub height: f32,
}

pub struct InlineFragment {
    pub x: f32,
    pub width: f32,
    pub layout_box_index: usize,
}

impl InlineFormattingContext {
    pub fn new(max_width: f32, line_height: f32) -> Self {
        Self {
            line_boxes: Vec::new(),
            current_x: 0.0,
            current_y: 0.0,
            line_height,
            max_width,
        }
    }

    pub fn add_inline_box(&mut self, width: f32, height: f32, index: usize) {
        if self.current_x + width > self.max_width && self.current_x > 0.0 {
            self.new_line();
        }

        if self.line_boxes.is_empty() {
            self.line_boxes.push(LineBox {
                fragments: Vec::new(),
                y: self.current_y,
                height: self.line_height,
            });
        }

        let line = self.line_boxes.last_mut().unwrap();
        line.fragments.push(InlineFragment {
            x: self.current_x,
            width,
            layout_box_index: index,
        });

        if height > line.height {
            line.height = height;
        }

        self.current_x += width;
    }

    pub fn new_line(&mut self) {
        if let Some(line) = self.line_boxes.last() {
            self.current_y += line.height;
        }

        self.current_x = 0.0;
        self.line_boxes.push(LineBox {
            fragments: Vec::new(),
            y: self.current_y,
            height: self.line_height,
        });
    }

    pub fn total_height(&self) -> f32 {
        self.line_boxes
            .iter()
            .map(|l| l.height)
            .sum()
    }
}

pub fn layout_inline_children(
    children: &mut [LayoutBox],
    max_width: f32,
    line_height: f32,
    _text_renderer: &mut TextRenderer,
) -> f32 {
    let mut ctx = InlineFormattingContext::new(max_width, line_height);

    for (i, child) in children.iter_mut().enumerate() {
        match child.box_type {
            BoxType::Inline | BoxType::InlineBlock | BoxType::Text => {
                let width = child.dimensions.content.width;
                let height = child.dimensions.content.height;
                ctx.add_inline_box(width, height, i);
            }
            BoxType::Block => {
                if ctx.current_x > 0.0 {
                    ctx.new_line();
                }
                ctx.current_y += child.dimensions.margin_box().height;
                ctx.new_line();
            }
            _ => {}
        }
    }

    for line in &ctx.line_boxes {
        for fragment in &line.fragments {
            if let Some(child) = children.get_mut(fragment.layout_box_index) {
                child.dimensions.content.x = fragment.x;
                child.dimensions.content.y = line.y;
            }
        }
    }

    ctx.total_height()
}
