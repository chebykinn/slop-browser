use crate::render::painter::Rect;

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub fn uniform(size: f32) -> Self {
        Self {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoxDimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl BoxDimensions {
    pub fn new() -> Self {
        Self {
            content: Rect::new(0.0, 0.0, 0.0, 0.0),
            padding: EdgeSizes::default(),
            border: EdgeSizes::default(),
            margin: EdgeSizes::default(),
        }
    }

    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.horizontal(),
            height: self.content.height + self.padding.vertical(),
        }
    }

    pub fn border_box(&self) -> Rect {
        let padding = self.padding_box();
        Rect {
            x: padding.x - self.border.left,
            y: padding.y - self.border.top,
            width: padding.width + self.border.horizontal(),
            height: padding.height + self.border.vertical(),
        }
    }

    pub fn margin_box(&self) -> Rect {
        let border = self.border_box();
        Rect {
            x: border.x - self.margin.left,
            y: border.y - self.margin.top,
            width: border.width + self.margin.horizontal(),
            height: border.height + self.margin.vertical(),
        }
    }

    pub fn content_x(&self) -> f32 {
        self.content.x
    }

    pub fn content_y(&self) -> f32 {
        self.content.y
    }

    pub fn content_width(&self) -> f32 {
        self.content.width
    }

    pub fn content_height(&self) -> f32 {
        self.content.height
    }

    pub fn set_content_position(&mut self, x: f32, y: f32) {
        self.content.x = x;
        self.content.y = y;
    }

    pub fn set_content_size(&mut self, width: f32, height: f32) {
        self.content.width = width;
        self.content.height = height;
    }
}
