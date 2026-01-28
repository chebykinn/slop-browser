use super::stylesheet::Value;
use crate::render::painter::Color;

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    // Display
    pub display: Display,

    // Box model
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,

    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,

    pub border_top_width: f32,
    pub border_right_width: f32,
    pub border_bottom_width: f32,
    pub border_left_width: f32,
    pub border_color: Color,

    // Dimensions
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,

    // Colors
    pub color: Color,
    pub background_color: Color,

    // Text
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub line_height: f32,
    pub text_decoration: TextDecoration,

    // Position
    pub position: Position,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,

    // Flexbox container properties
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub gap: f32,
    pub row_gap: f32,
    pub column_gap: f32,

    // Flexbox item properties
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
    pub align_self: Option<AlignItems>,
    pub order: i32,

    // Grid container properties
    pub grid_template_columns: Vec<GridTrackSize>,
    pub grid_template_rows: Vec<GridTrackSize>,
    pub grid_auto_columns: GridTrackSize,
    pub grid_auto_rows: GridTrackSize,
    pub grid_gap: f32,
    pub grid_row_gap: f32,
    pub grid_column_gap: f32,

    // Grid item properties
    pub grid_column: GridPlacement,
    pub grid_row: GridPlacement,

    // Visual properties
    pub border_radius: f32,
    pub border_top_left_radius: f32,
    pub border_top_right_radius: f32,
    pub border_bottom_left_radius: f32,
    pub border_bottom_right_radius: f32,
    pub box_shadow: Option<BoxShadow>,
    pub opacity: f32,
    pub overflow: Overflow,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    // Box sizing
    pub box_sizing: BoxSizing,

    // Text handling
    pub white_space: WhiteSpace,
    pub vertical_align: VerticalAlign,

    // Visibility and stacking
    pub visibility: Visibility,
    pub z_index: Option<i32>,

    // Border styles
    pub border_style: BorderStyle,
    pub border_top_style: BorderStyle,
    pub border_right_style: BorderStyle,
    pub border_bottom_style: BorderStyle,
    pub border_left_style: BorderStyle,

    // Table-specific properties
    pub border_collapse: bool,
    pub border_spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
    Grid,
    Table,
    TableRow,
    TableCell,
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableCaption,
    TableColumn,
    TableColumnGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

/// Grid track sizing value
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrackSize {
    /// Fixed pixel size
    Px(f32),
    /// Fractional unit (fr)
    Fr(f32),
    /// Auto sizing
    Auto,
    /// min-content
    MinContent,
    /// max-content
    MaxContent,
}

impl Default for GridTrackSize {
    fn default() -> Self {
        GridTrackSize::Auto
    }
}

/// Grid placement (start/end line)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GridPlacement {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub span: Option<u32>,
}

/// Box shadow definition
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: crate::render::painter::Color,
    pub inset: bool,
}

/// Overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Box sizing model
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

/// White space handling
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
    TextTop,
    TextBottom,
    Sub,
    Super,
}

/// Visibility
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
    Numeric(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
    LineThrough,
    Overline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Block,

            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,

            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,

            border_top_width: 0.0,
            border_right_width: 0.0,
            border_bottom_width: 0.0,
            border_left_width: 0.0,
            border_color: Color::BLACK,

            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,

            color: Color::BLACK,
            background_color: Color::TRANSPARENT,

            font_size: 16.0,
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            line_height: 1.2,
            text_decoration: TextDecoration::None,

            position: Position::Static,
            top: None,
            right: None,
            bottom: None,
            left: None,

            // Flexbox container properties
            flex_direction: FlexDirection::default(),
            flex_wrap: FlexWrap::default(),
            justify_content: JustifyContent::default(),
            align_items: AlignItems::default(),
            align_content: AlignContent::default(),
            gap: 0.0,
            row_gap: 0.0,
            column_gap: 0.0,

            // Flexbox item properties
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            align_self: None,
            order: 0,

            // Grid container properties
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_auto_columns: GridTrackSize::Auto,
            grid_auto_rows: GridTrackSize::Auto,
            grid_gap: 0.0,
            grid_row_gap: 0.0,
            grid_column_gap: 0.0,

            // Grid item properties
            grid_column: GridPlacement::default(),
            grid_row: GridPlacement::default(),

            // Visual properties
            border_radius: 0.0,
            border_top_left_radius: 0.0,
            border_top_right_radius: 0.0,
            border_bottom_left_radius: 0.0,
            border_bottom_right_radius: 0.0,
            box_shadow: None,
            opacity: 1.0,
            overflow: Overflow::default(),
            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),

            // Box sizing
            box_sizing: BoxSizing::default(),

            // Text handling
            white_space: WhiteSpace::default(),
            vertical_align: VerticalAlign::default(),

            // Visibility and stacking
            visibility: Visibility::default(),
            z_index: None,

            // Border styles
            border_style: BorderStyle::default(),
            border_top_style: BorderStyle::default(),
            border_right_style: BorderStyle::default(),
            border_bottom_style: BorderStyle::default(),
            border_left_style: BorderStyle::default(),

            // Table-specific properties
            border_collapse: false,
            border_spacing: 0.0,
        }
    }
}

impl ComputedStyle {
    pub fn apply_value(&mut self, property: &str, value: &Value, parent_font_size: f32, vw: f32, vh: f32) {
        match property {
            "display" => {
                if let Some(kw) = value.as_keyword() {
                    self.display = match kw {
                        "block" => Display::Block,
                        "inline" => Display::Inline,
                        "inline-block" => Display::InlineBlock,
                        "none" => Display::None,
                        "flex" => Display::Flex,
                        "grid" => Display::Grid,
                        "table" => Display::Table,
                        "table-row" => Display::TableRow,
                        "table-cell" => Display::TableCell,
                        "table-row-group" => Display::TableRowGroup,
                        "table-header-group" => Display::TableHeaderGroup,
                        "table-footer-group" => Display::TableFooterGroup,
                        "table-caption" => Display::TableCaption,
                        "table-column" => Display::TableColumn,
                        "table-column-group" => Display::TableColumnGroup,
                        _ => Display::Block,
                    };
                }
            }

            "color" => {
                if let Some(c) = value.to_color() {
                    self.color = c;
                }
            }

            "background-color" | "background" => {
                if let Some(c) = value.to_color() {
                    self.background_color = c;
                }
            }

            "font-size" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.font_size = px;
                }
            }

            "font-weight" => {
                if let Some(kw) = value.as_keyword() {
                    self.font_weight = match kw {
                        "bold" => FontWeight::Bold,
                        "normal" => FontWeight::Normal,
                        _ => FontWeight::Normal,
                    };
                } else if let Value::Number(n) = value {
                    self.font_weight = FontWeight::Numeric(*n as u16);
                }
            }

            "text-align" => {
                if let Some(kw) = value.as_keyword() {
                    self.text_align = match kw {
                        "left" => TextAlign::Left,
                        "center" => TextAlign::Center,
                        "right" => TextAlign::Right,
                        "justify" => TextAlign::Justify,
                        _ => TextAlign::Left,
                    };
                }
            }

            "text-decoration" => {
                if let Some(kw) = value.as_keyword() {
                    self.text_decoration = match kw {
                        "none" => TextDecoration::None,
                        "underline" => TextDecoration::Underline,
                        "line-through" => TextDecoration::LineThrough,
                        "overline" => TextDecoration::Overline,
                        _ => TextDecoration::None,
                    };
                }
            }

            "line-height" => {
                if let Value::Number(n) = value {
                    self.line_height = *n;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.line_height = px / self.font_size;
                }
            }

            "margin" => {
                match value {
                    Value::List(values) => {
                        let vals: Vec<f32> = values
                            .iter()
                            .filter_map(|v| v.to_px(parent_font_size, vw, vh))
                            .collect();
                        match vals.len() {
                            1 => {
                                self.margin_top = vals[0];
                                self.margin_right = vals[0];
                                self.margin_bottom = vals[0];
                                self.margin_left = vals[0];
                            }
                            2 => {
                                self.margin_top = vals[0];
                                self.margin_bottom = vals[0];
                                self.margin_right = vals[1];
                                self.margin_left = vals[1];
                            }
                            3 => {
                                self.margin_top = vals[0];
                                self.margin_right = vals[1];
                                self.margin_left = vals[1];
                                self.margin_bottom = vals[2];
                            }
                            4 => {
                                self.margin_top = vals[0];
                                self.margin_right = vals[1];
                                self.margin_bottom = vals[2];
                                self.margin_left = vals[3];
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                            self.margin_top = px;
                            self.margin_right = px;
                            self.margin_bottom = px;
                            self.margin_left = px;
                        }
                    }
                }
            }
            "margin-top" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.margin_top = px;
                }
            }
            "margin-right" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.margin_right = px;
                }
            }
            "margin-bottom" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.margin_bottom = px;
                }
            }
            "margin-left" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.margin_left = px;
                }
            }

            "padding" => {
                match value {
                    Value::List(values) => {
                        let vals: Vec<f32> = values
                            .iter()
                            .filter_map(|v| v.to_px(parent_font_size, vw, vh))
                            .collect();
                        match vals.len() {
                            1 => {
                                self.padding_top = vals[0];
                                self.padding_right = vals[0];
                                self.padding_bottom = vals[0];
                                self.padding_left = vals[0];
                            }
                            2 => {
                                self.padding_top = vals[0];
                                self.padding_bottom = vals[0];
                                self.padding_right = vals[1];
                                self.padding_left = vals[1];
                            }
                            3 => {
                                self.padding_top = vals[0];
                                self.padding_right = vals[1];
                                self.padding_left = vals[1];
                                self.padding_bottom = vals[2];
                            }
                            4 => {
                                self.padding_top = vals[0];
                                self.padding_right = vals[1];
                                self.padding_bottom = vals[2];
                                self.padding_left = vals[3];
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                            self.padding_top = px;
                            self.padding_right = px;
                            self.padding_bottom = px;
                            self.padding_left = px;
                        }
                    }
                }
            }
            "padding-top" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.padding_top = px;
                }
            }
            "padding-right" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.padding_right = px;
                }
            }
            "padding-bottom" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.padding_bottom = px;
                }
            }
            "padding-left" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.padding_left = px;
                }
            }

            "border-width" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_top_width = px;
                    self.border_right_width = px;
                    self.border_bottom_width = px;
                    self.border_left_width = px;
                }
            }
            "border-color" => {
                if let Some(c) = value.to_color() {
                    self.border_color = c;
                }
            }

            "width" => {
                if let Value::Auto = value {
                    self.width = None;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.width = Some(px);
                }
            }
            "height" => {
                if let Value::Auto = value {
                    self.height = None;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.height = Some(px);
                }
            }

            "position" => {
                if let Some(kw) = value.as_keyword() {
                    self.position = match kw {
                        "static" => Position::Static,
                        "relative" => Position::Relative,
                        "absolute" => Position::Absolute,
                        "fixed" => Position::Fixed,
                        _ => Position::Static,
                    };
                }
            }

            "top" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.top = Some(px);
                }
            }
            "right" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.right = Some(px);
                }
            }
            "bottom" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.bottom = Some(px);
                }
            }
            "left" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.left = Some(px);
                }
            }

            // Flexbox container properties
            "flex-direction" => {
                if let Some(kw) = value.as_keyword() {
                    self.flex_direction = match kw {
                        "row" => FlexDirection::Row,
                        "row-reverse" => FlexDirection::RowReverse,
                        "column" => FlexDirection::Column,
                        "column-reverse" => FlexDirection::ColumnReverse,
                        _ => FlexDirection::Row,
                    };
                }
            }

            "flex-wrap" => {
                if let Some(kw) = value.as_keyword() {
                    self.flex_wrap = match kw {
                        "nowrap" => FlexWrap::NoWrap,
                        "wrap" => FlexWrap::Wrap,
                        "wrap-reverse" => FlexWrap::WrapReverse,
                        _ => FlexWrap::NoWrap,
                    };
                }
            }

            "justify-content" => {
                if let Some(kw) = value.as_keyword() {
                    self.justify_content = match kw {
                        "flex-start" | "start" => JustifyContent::FlexStart,
                        "flex-end" | "end" => JustifyContent::FlexEnd,
                        "center" => JustifyContent::Center,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        "space-evenly" => JustifyContent::SpaceEvenly,
                        _ => JustifyContent::FlexStart,
                    };
                }
            }

            "align-items" => {
                if let Some(kw) = value.as_keyword() {
                    self.align_items = match kw {
                        "flex-start" | "start" => AlignItems::FlexStart,
                        "flex-end" | "end" => AlignItems::FlexEnd,
                        "center" => AlignItems::Center,
                        "stretch" => AlignItems::Stretch,
                        "baseline" => AlignItems::Baseline,
                        _ => AlignItems::Stretch,
                    };
                }
            }

            "align-content" => {
                if let Some(kw) = value.as_keyword() {
                    self.align_content = match kw {
                        "flex-start" | "start" => AlignContent::FlexStart,
                        "flex-end" | "end" => AlignContent::FlexEnd,
                        "center" => AlignContent::Center,
                        "space-between" => AlignContent::SpaceBetween,
                        "space-around" => AlignContent::SpaceAround,
                        "stretch" => AlignContent::Stretch,
                        _ => AlignContent::FlexStart,
                    };
                }
            }

            "gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.gap = px;
                    self.row_gap = px;
                    self.column_gap = px;
                }
            }

            "row-gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.row_gap = px;
                }
            }

            "column-gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.column_gap = px;
                }
            }

            // Flexbox item properties
            "flex-grow" => {
                if let Value::Number(n) = value {
                    self.flex_grow = *n;
                }
            }

            "flex-shrink" => {
                if let Value::Number(n) = value {
                    self.flex_shrink = *n;
                }
            }

            "flex-basis" => {
                if let Value::Auto = value {
                    self.flex_basis = None;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.flex_basis = Some(px);
                }
            }

            "flex" => {
                // Shorthand: flex: <grow> <shrink>? <basis>?
                // Common values: flex: 1 means flex: 1 1 0
                if let Value::Number(n) = value {
                    self.flex_grow = *n;
                    self.flex_shrink = 1.0;
                    self.flex_basis = Some(0.0);
                } else if let Some(kw) = value.as_keyword() {
                    match kw {
                        "auto" => {
                            self.flex_grow = 1.0;
                            self.flex_shrink = 1.0;
                            self.flex_basis = None;
                        }
                        "none" => {
                            self.flex_grow = 0.0;
                            self.flex_shrink = 0.0;
                            self.flex_basis = None;
                        }
                        _ => {}
                    }
                }
            }

            "align-self" => {
                if let Some(kw) = value.as_keyword() {
                    self.align_self = Some(match kw {
                        "flex-start" | "start" => AlignItems::FlexStart,
                        "flex-end" | "end" => AlignItems::FlexEnd,
                        "center" => AlignItems::Center,
                        "stretch" => AlignItems::Stretch,
                        "baseline" => AlignItems::Baseline,
                        "auto" => return, // auto means inherit from parent
                        _ => AlignItems::Stretch,
                    });
                }
            }

            "order" => {
                if let Value::Number(n) = value {
                    self.order = *n as i32;
                }
            }

            // Grid container properties
            "grid-template-columns" => {
                // Parse is handled separately for complex track lists
                // For simple cases, parse keywords
                if let Some(kw) = value.as_keyword() {
                    if kw == "none" {
                        self.grid_template_columns.clear();
                    }
                }
            }

            "grid-template-rows" => {
                if let Some(kw) = value.as_keyword() {
                    if kw == "none" {
                        self.grid_template_rows.clear();
                    }
                }
            }

            "grid-gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.grid_gap = px;
                    self.grid_row_gap = px;
                    self.grid_column_gap = px;
                }
            }

            "grid-row-gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.grid_row_gap = px;
                }
            }

            "grid-column-gap" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.grid_column_gap = px;
                }
            }

            "grid-column-start" => {
                if let Value::Number(n) = value {
                    self.grid_column.start = Some(*n as i32);
                }
            }

            "grid-column-end" => {
                if let Value::Number(n) = value {
                    self.grid_column.end = Some(*n as i32);
                }
            }

            "grid-row-start" => {
                if let Value::Number(n) = value {
                    self.grid_row.start = Some(*n as i32);
                }
            }

            "grid-row-end" => {
                if let Value::Number(n) = value {
                    self.grid_row.end = Some(*n as i32);
                }
            }

            // Visual properties
            "border-radius" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_radius = px;
                    self.border_top_left_radius = px;
                    self.border_top_right_radius = px;
                    self.border_bottom_left_radius = px;
                    self.border_bottom_right_radius = px;
                }
            }

            "border-top-left-radius" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_top_left_radius = px;
                }
            }

            "border-top-right-radius" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_top_right_radius = px;
                }
            }

            "border-bottom-left-radius" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_bottom_left_radius = px;
                }
            }

            "border-bottom-right-radius" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_bottom_right_radius = px;
                }
            }

            "opacity" => {
                if let Value::Number(n) = value {
                    self.opacity = n.clamp(0.0, 1.0);
                }
            }

            "overflow" => {
                if let Some(kw) = value.as_keyword() {
                    let overflow = match kw {
                        "visible" => Overflow::Visible,
                        "hidden" => Overflow::Hidden,
                        "scroll" => Overflow::Scroll,
                        "auto" => Overflow::Auto,
                        _ => Overflow::Visible,
                    };
                    self.overflow = overflow;
                    self.overflow_x = overflow;
                    self.overflow_y = overflow;
                }
            }

            "overflow-x" => {
                if let Some(kw) = value.as_keyword() {
                    self.overflow_x = match kw {
                        "visible" => Overflow::Visible,
                        "hidden" => Overflow::Hidden,
                        "scroll" => Overflow::Scroll,
                        "auto" => Overflow::Auto,
                        _ => Overflow::Visible,
                    };
                }
            }

            "overflow-y" => {
                if let Some(kw) = value.as_keyword() {
                    self.overflow_y = match kw {
                        "visible" => Overflow::Visible,
                        "hidden" => Overflow::Hidden,
                        "scroll" => Overflow::Scroll,
                        "auto" => Overflow::Auto,
                        _ => Overflow::Visible,
                    };
                }
            }

            // Box sizing
            "box-sizing" => {
                if let Some(kw) = value.as_keyword() {
                    self.box_sizing = match kw {
                        "content-box" => BoxSizing::ContentBox,
                        "border-box" => BoxSizing::BorderBox,
                        _ => BoxSizing::ContentBox,
                    };
                }
            }

            // White space
            "white-space" => {
                if let Some(kw) = value.as_keyword() {
                    self.white_space = match kw {
                        "normal" => WhiteSpace::Normal,
                        "nowrap" => WhiteSpace::NoWrap,
                        "pre" => WhiteSpace::Pre,
                        "pre-wrap" => WhiteSpace::PreWrap,
                        "pre-line" => WhiteSpace::PreLine,
                        _ => WhiteSpace::Normal,
                    };
                }
            }

            // Vertical align
            "vertical-align" => {
                if let Some(kw) = value.as_keyword() {
                    self.vertical_align = match kw {
                        "baseline" => VerticalAlign::Baseline,
                        "top" => VerticalAlign::Top,
                        "middle" => VerticalAlign::Middle,
                        "bottom" => VerticalAlign::Bottom,
                        "text-top" => VerticalAlign::TextTop,
                        "text-bottom" => VerticalAlign::TextBottom,
                        "sub" => VerticalAlign::Sub,
                        "super" => VerticalAlign::Super,
                        _ => VerticalAlign::Baseline,
                    };
                }
            }

            // Visibility
            "visibility" => {
                if let Some(kw) = value.as_keyword() {
                    self.visibility = match kw {
                        "visible" => Visibility::Visible,
                        "hidden" => Visibility::Hidden,
                        "collapse" => Visibility::Collapse,
                        _ => Visibility::Visible,
                    };
                }
            }

            // Z-index
            "z-index" => {
                if let Value::Number(n) = value {
                    self.z_index = Some(*n as i32);
                } else if let Some(kw) = value.as_keyword() {
                    if kw == "auto" {
                        self.z_index = None;
                    }
                }
            }

            // Min/max dimensions
            "min-width" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.min_width = Some(px);
                }
            }
            "max-width" => {
                if let Value::None = value {
                    self.max_width = None;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.max_width = Some(px);
                }
            }
            "min-height" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.min_height = Some(px);
                }
            }
            "max-height" => {
                if let Value::None = value {
                    self.max_height = None;
                } else if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.max_height = Some(px);
                }
            }

            // Border shorthand: border: <width> <style> <color>
            "border" => {
                self.apply_border_shorthand(value, parent_font_size, vw, vh);
            }
            "border-top" => {
                if let Some((width, style, color)) = Self::parse_border_shorthand(value, parent_font_size, vw, vh) {
                    if let Some(w) = width { self.border_top_width = w; }
                    if let Some(s) = style { self.border_top_style = s; }
                    if let Some(c) = color { self.border_color = c; }
                }
            }
            "border-right" => {
                if let Some((width, style, color)) = Self::parse_border_shorthand(value, parent_font_size, vw, vh) {
                    if let Some(w) = width { self.border_right_width = w; }
                    if let Some(s) = style { self.border_right_style = s; }
                    if let Some(c) = color { self.border_color = c; }
                }
            }
            "border-bottom" => {
                if let Some((width, style, color)) = Self::parse_border_shorthand(value, parent_font_size, vw, vh) {
                    if let Some(w) = width { self.border_bottom_width = w; }
                    if let Some(s) = style { self.border_bottom_style = s; }
                    if let Some(c) = color { self.border_color = c; }
                }
            }
            "border-left" => {
                if let Some((width, style, color)) = Self::parse_border_shorthand(value, parent_font_size, vw, vh) {
                    if let Some(w) = width { self.border_left_width = w; }
                    if let Some(s) = style { self.border_left_style = s; }
                    if let Some(c) = color { self.border_color = c; }
                }
            }
            "border-style" => {
                if let Some(kw) = value.as_keyword() {
                    let style = Self::parse_border_style(kw);
                    self.border_style = style;
                    self.border_top_style = style;
                    self.border_right_style = style;
                    self.border_bottom_style = style;
                    self.border_left_style = style;
                }
            }

            // Table-specific properties
            "border-collapse" => {
                if let Some(kw) = value.as_keyword() {
                    self.border_collapse = kw == "collapse";
                }
            }
            "border-spacing" => {
                if let Some(px) = value.to_px(parent_font_size, vw, vh) {
                    self.border_spacing = px;
                }
            }

            _ => {}
        }
    }

    fn apply_border_shorthand(&mut self, value: &Value, parent_font_size: f32, vw: f32, vh: f32) {
        if let Some((width, style, color)) = Self::parse_border_shorthand(value, parent_font_size, vw, vh) {
            if let Some(w) = width {
                self.border_top_width = w;
                self.border_right_width = w;
                self.border_bottom_width = w;
                self.border_left_width = w;
            }
            if let Some(s) = style {
                self.border_style = s;
                self.border_top_style = s;
                self.border_right_style = s;
                self.border_bottom_style = s;
                self.border_left_style = s;
            }
            if let Some(c) = color {
                self.border_color = c;
            }
        }
    }

    fn parse_border_shorthand(value: &Value, parent_font_size: f32, vw: f32, vh: f32) -> Option<(Option<f32>, Option<BorderStyle>, Option<Color>)> {
        let mut width = None;
        let mut style = None;
        let mut color = None;

        let values = match value {
            Value::List(vs) => vs.as_slice(),
            _ => std::slice::from_ref(value),
        };

        for v in values {
            // Try as border style first (keywords like solid, dashed)
            if let Some(kw) = v.as_keyword() {
                let parsed_style = Self::parse_border_style(kw);
                if parsed_style != BorderStyle::None || kw == "none" {
                    style = Some(parsed_style);
                    continue;
                }
            }

            // Try as color
            if let Some(c) = v.to_color() {
                color = Some(c);
                continue;
            }

            // Try as width
            if let Some(px) = v.to_px(parent_font_size, vw, vh) {
                width = Some(px);
                continue;
            }

            // Handle keyword widths
            if let Some(kw) = v.as_keyword() {
                match kw {
                    "thin" => { width = Some(1.0); continue; }
                    "medium" => { width = Some(3.0); continue; }
                    "thick" => { width = Some(5.0); continue; }
                    _ => {}
                }
            }
        }

        Some((width, style, color))
    }

    fn parse_border_style(kw: &str) -> BorderStyle {
        match kw {
            "none" => BorderStyle::None,
            "solid" => BorderStyle::Solid,
            "dashed" => BorderStyle::Dashed,
            "dotted" => BorderStyle::Dotted,
            "double" => BorderStyle::Double,
            "groove" => BorderStyle::Groove,
            "ridge" => BorderStyle::Ridge,
            "inset" => BorderStyle::Inset,
            "outset" => BorderStyle::Outset,
            "hidden" => BorderStyle::Hidden,
            _ => BorderStyle::None,
        }
    }

    pub fn for_tag(tag: &str) -> Self {
        let mut style = Self::default();

        match tag {
            "h1" => {
                style.font_size = 32.0;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 21.44;
                style.margin_bottom = 21.44;
            }
            "h2" => {
                style.font_size = 24.0;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 19.92;
                style.margin_bottom = 19.92;
            }
            "h3" => {
                style.font_size = 18.72;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 18.72;
                style.margin_bottom = 18.72;
            }
            "h4" => {
                style.font_size = 16.0;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 21.28;
                style.margin_bottom = 21.28;
            }
            "h5" => {
                style.font_size = 13.28;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 22.17;
                style.margin_bottom = 22.17;
            }
            "h6" => {
                style.font_size = 10.72;
                style.font_weight = FontWeight::Bold;
                style.margin_top = 24.97;
                style.margin_bottom = 24.97;
            }
            "p" => {
                style.margin_top = 16.0;
                style.margin_bottom = 16.0;
            }
            "a" => {
                style.display = Display::Inline;
                style.color = Color::rgb(0, 0, 238);
                style.text_decoration = TextDecoration::Underline;
            }
            "strong" | "b" => {
                style.display = Display::Inline;
                style.font_weight = FontWeight::Bold;
            }
            "em" | "i" => {
                style.display = Display::Inline;
            }
            "span" => {
                style.display = Display::Inline;
            }
            "div" => {
                style.display = Display::Block;
            }
            "ul" | "ol" => {
                style.margin_top = 16.0;
                style.margin_bottom = 16.0;
                style.padding_left = 40.0;
            }
            "li" => {
                style.display = Display::Block;
            }
            "img" => {
                style.display = Display::InlineBlock;
            }
            "br" => {
                style.display = Display::Block;
            }
            // Table elements
            "table" => {
                style.display = Display::Table;
                style.border_collapse = false;
                style.border_spacing = 2.0;
            }
            "tr" => {
                style.display = Display::TableRow;
            }
            "td" => {
                style.display = Display::TableCell;
                style.padding_top = 1.0;
                style.padding_right = 1.0;
                style.padding_bottom = 1.0;
                style.padding_left = 1.0;
            }
            "th" => {
                style.display = Display::TableCell;
                style.font_weight = FontWeight::Bold;
                style.text_align = TextAlign::Center;
                style.padding_top = 1.0;
                style.padding_right = 1.0;
                style.padding_bottom = 1.0;
                style.padding_left = 1.0;
            }
            "thead" => {
                style.display = Display::TableHeaderGroup;
            }
            "tbody" => {
                style.display = Display::TableRowGroup;
            }
            "tfoot" => {
                style.display = Display::TableFooterGroup;
            }
            "caption" => {
                style.display = Display::TableCaption;
                style.text_align = TextAlign::Center;
            }
            "col" => {
                style.display = Display::TableColumn;
            }
            "colgroup" => {
                style.display = Display::TableColumnGroup;
            }
            // Form elements
            "input" | "button" | "select" | "textarea" => {
                style.display = Display::InlineBlock;
            }
            _ => {}
        }

        style
    }
}
