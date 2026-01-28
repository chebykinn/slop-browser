use super::box_model::{BoxDimensions, EdgeSizes};
use crate::css::computed::{BoxSizing, ComputedStyle, Display};
use crate::css::StyleComputer;
use crate::dom::{Document, NodeData, NodeId};
use crate::render::painter::{Color, DisplayList, Rect};
use crate::render::text::TextRenderer;
use crate::render::ImageSize;

/// Handles smooth scroll animation with momentum
#[derive(Debug, Clone)]
pub struct ScrollAnimator {
    pub current: f32,
    pub target: f32,
    pub animating: bool,
}

impl Default for ScrollAnimator {
    fn default() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            animating: false,
        }
    }
}

impl ScrollAnimator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, max_scroll: f32) -> bool {
        if !self.animating {
            return false;
        }

        self.target = self.target.clamp(0.0, max_scroll);

        let lerp_speed = 12.0;
        let diff = self.target - self.current;

        if diff.abs() < 0.5 {
            self.current = self.target;
            self.animating = false;
            return false;
        }

        self.current += diff * (1.0 - (-lerp_speed * dt).exp());
        self.current = self.current.clamp(0.0, max_scroll);

        true
    }

    pub fn scroll_by(&mut self, delta: f32, max_scroll: f32) {
        self.target = (self.target + delta).clamp(0.0, max_scroll);
        self.animating = true;
    }

    pub fn scroll_to(&mut self, position: f32, max_scroll: f32) {
        self.target = position.clamp(0.0, max_scroll);
        self.animating = true;
    }

    pub fn scroll_immediate(&mut self, position: f32, max_scroll: f32) {
        self.current = position.clamp(0.0, max_scroll);
        self.target = self.current;
        self.animating = false;
    }
}

#[derive(Debug, Clone)]
pub struct ScrollbarConfig {
    pub width: f32,
    pub min_thumb_height: f32,
    pub track_color: Color,
    pub thumb_color: Color,
    pub thumb_hover_color: Color,
    pub thumb_active_color: Color,
}

impl Default for ScrollbarConfig {
    fn default() -> Self {
        Self {
            width: 12.0,
            min_thumb_height: 30.0,
            track_color: Color::rgba(200, 200, 200, 128),
            thumb_color: Color::rgba(120, 120, 120, 180),
            thumb_hover_color: Color::rgba(100, 100, 100, 200),
            thumb_active_color: Color::rgba(80, 80, 80, 220),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScrollbarState {
    Idle,
    Hovering,
    Dragging { start_y: f32, start_scroll: f32 },
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollbarHitArea {
    None,
    Track,
    Thumb,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    InlineBlock,
    Anonymous,
    Text,
    Flex,
    Grid,
    Image,
    Table,
    TableRow,
    TableCell,
    TableRowGroup,
}

pub struct LayoutBox {
    pub box_type: BoxType,
    pub dimensions: BoxDimensions,
    pub node_id: Option<NodeId>,
    pub children: Vec<LayoutBox>,
    pub text_content: Option<String>,
    pub style: ComputedStyle,
    /// Image source URL (for img elements)
    pub image_src: Option<String>,
    /// Intrinsic image dimensions (for img elements)
    pub intrinsic_size: Option<ImageSize>,
    /// Texture ID for loaded images
    pub texture_id: Option<usize>,
}

impl LayoutBox {
    pub fn new(box_type: BoxType, node_id: Option<NodeId>) -> Self {
        Self {
            box_type,
            dimensions: BoxDimensions::new(),
            node_id,
            children: Vec::new(),
            text_content: None,
            style: ComputedStyle::default(),
            image_src: None,
            intrinsic_size: None,
            texture_id: None,
        }
    }

    pub fn new_text(text: String, style: ComputedStyle) -> Self {
        Self {
            box_type: BoxType::Text,
            dimensions: BoxDimensions::new(),
            node_id: None,
            children: Vec::new(),
            text_content: Some(text),
            style,
            image_src: None,
            intrinsic_size: None,
            texture_id: None,
        }
    }

    pub fn new_image(src: String, style: ComputedStyle, node_id: Option<NodeId>) -> Self {
        Self {
            box_type: BoxType::Image,
            dimensions: BoxDimensions::new(),
            node_id,
            children: Vec::new(),
            text_content: None,
            style,
            image_src: Some(src),
            intrinsic_size: None,
            texture_id: None,
        }
    }
}

pub struct LayoutTree {
    pub root: Option<LayoutBox>,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub scroll_y: f32,
    pub scroll_animator: ScrollAnimator,
    pub scrollbar_config: ScrollbarConfig,
    pub scrollbar_state: ScrollbarState,
}

impl LayoutTree {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            root: None,
            viewport_width,
            viewport_height,
            scroll_y: 0.0,
            scroll_animator: ScrollAnimator::new(),
            scrollbar_config: ScrollbarConfig::default(),
            scrollbar_state: ScrollbarState::default(),
        }
    }

    pub fn build(
        &mut self,
        document: &Document,
        style_computer: &StyleComputer,
        text_renderer: &mut TextRenderer,
    ) {
        use std::time::Instant;

        if let Some(body) = document.get_body() {
            let t0 = Instant::now();
            let mut root_box = self.build_layout_tree(document, body, style_computer);
            let t1 = Instant::now();
            self.layout(&mut root_box, self.viewport_width, text_renderer);
            let t2 = Instant::now();

            // Position root so border_box starts at (0, 0)
            root_box.dimensions.content.x = root_box.dimensions.margin.left
                + root_box.dimensions.border.left
                + root_box.dimensions.padding.left;
            root_box.dimensions.content.y = root_box.dimensions.margin.top
                + root_box.dimensions.border.top
                + root_box.dimensions.padding.top;

            self.root = Some(root_box);

            println!(
                "[Layout breakdown] build_tree={:.2}ms layout={:.2}ms",
                (t1 - t0).as_secs_f32() * 1000.0,
                (t2 - t1).as_secs_f32() * 1000.0,
            );
        }
    }

    fn build_layout_tree(
        &self,
        document: &Document,
        node_id: NodeId,
        style_computer: &StyleComputer,
    ) -> LayoutBox {
        let node = document.get_node(node_id).unwrap();
        let style = style_computer
            .get_style(node_id)
            .cloned()
            .unwrap_or_default();

        match &node.data {
            NodeData::Element(elem) => {
                let tag = &elem.tag_name;

                // Skip script and style elements - they should not be rendered
                if tag == "script" || tag == "style" {
                    return LayoutBox::new(BoxType::Block, None);
                }

                // Handle img elements specially
                if tag == "img" {
                    if let Some(src) = elem.get_attribute("src") {
                        let mut layout_box = LayoutBox::new_image(src.to_string(), style, Some(node_id));

                        // Try to get width/height attributes as hints
                        if let Some(w) = elem.get_attribute("width") {
                            if let Ok(width) = w.parse::<u32>() {
                                if layout_box.intrinsic_size.is_none() {
                                    layout_box.intrinsic_size = Some(ImageSize::new(width, 0));
                                }
                                if let Some(ref mut size) = layout_box.intrinsic_size {
                                    size.width = width;
                                }
                            }
                        }
                        if let Some(h) = elem.get_attribute("height") {
                            if let Ok(height) = h.parse::<u32>() {
                                if let Some(ref mut size) = layout_box.intrinsic_size {
                                    size.height = height;
                                } else {
                                    layout_box.intrinsic_size = Some(ImageSize::new(0, height));
                                }
                            }
                        }

                        return layout_box;
                    } else {
                        // img without src - render nothing
                        return LayoutBox::new(BoxType::Block, None);
                    }
                }

                let box_type = match style.display {
                    Display::Block => BoxType::Block,
                    Display::Inline => BoxType::Inline,
                    Display::InlineBlock => BoxType::InlineBlock,
                    Display::None => return LayoutBox::new(BoxType::Block, Some(node_id)),
                    Display::Flex => BoxType::Flex,
                    Display::Grid => BoxType::Grid,
                    Display::Table => BoxType::Table,
                    Display::TableRow => BoxType::TableRow,
                    Display::TableCell => BoxType::TableCell,
                    Display::TableRowGroup | Display::TableHeaderGroup | Display::TableFooterGroup => BoxType::TableRowGroup,
                    Display::TableCaption | Display::TableColumn | Display::TableColumnGroup => BoxType::Block,
                };

                let mut layout_box = LayoutBox::new(box_type, Some(node_id));
                layout_box.style = style;

                for &child_id in document.children(node_id) {
                    let _child_node = document.get_node(child_id).unwrap();
                    let child_style = style_computer.get_style(child_id);

                    if let Some(s) = child_style {
                        if s.display == Display::None {
                            continue;
                        }
                    }

                    let child_box = self.build_layout_tree(document, child_id, style_computer);
                    layout_box.children.push(child_box);
                }

                layout_box
            }
            NodeData::Text(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    LayoutBox::new(BoxType::Text, None)
                } else {
                    LayoutBox::new_text(trimmed.to_string(), style)
                }
            }
            _ => LayoutBox::new(BoxType::Block, None),
        }
    }

    fn layout(&self, layout_box: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
        match layout_box.box_type {
            BoxType::Block | BoxType::InlineBlock => {
                self.layout_block(layout_box, containing_width, text_renderer);
            }
            BoxType::Inline => {
                self.layout_inline(layout_box, containing_width, text_renderer);
            }
            BoxType::Text => {
                self.layout_text(layout_box, containing_width, text_renderer);
            }
            BoxType::Anonymous => {
                self.layout_block(layout_box, containing_width, text_renderer);
            }
            BoxType::Flex => {
                super::flex::layout_flex(layout_box, containing_width, text_renderer);
            }
            BoxType::Grid => {
                super::grid::layout_grid(layout_box, containing_width, text_renderer);
            }
            BoxType::Image => {
                self.layout_image(layout_box, containing_width);
            }
            BoxType::Table => {
                super::table::layout_table(layout_box, containing_width, text_renderer);
            }
            BoxType::TableRow | BoxType::TableCell | BoxType::TableRowGroup => {
                // These are laid out by their parent table
                self.layout_block(layout_box, containing_width, text_renderer);
            }
        }
    }

    fn layout_block(&self, layout_box: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
        let style = &layout_box.style;

        layout_box.dimensions.margin = EdgeSizes::new(
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
        );
        layout_box.dimensions.padding = EdgeSizes::new(
            style.padding_top,
            style.padding_right,
            style.padding_bottom,
            style.padding_left,
        );
        layout_box.dimensions.border = EdgeSizes::new(
            style.border_top_width,
            style.border_right_width,
            style.border_bottom_width,
            style.border_left_width,
        );

        // Calculate content width based on box-sizing
        let content_width = match style.width {
            Some(specified_width) => {
                match style.box_sizing {
                    BoxSizing::ContentBox => specified_width,
                    BoxSizing::BorderBox => {
                        // border-box: specified width includes padding and border
                        (specified_width
                            - layout_box.dimensions.padding.horizontal()
                            - layout_box.dimensions.border.horizontal())
                        .max(0.0)
                    }
                }
            }
            None => {
                containing_width
                    - layout_box.dimensions.margin.horizontal()
                    - layout_box.dimensions.padding.horizontal()
                    - layout_box.dimensions.border.horizontal()
            }
        };

        // Apply min/max width constraints
        let mut final_width = content_width;
        if let Some(min_width) = style.min_width {
            final_width = final_width.max(min_width);
        }
        if let Some(max_width) = style.max_width {
            final_width = final_width.min(max_width);
        }

        layout_box.dimensions.content.width = final_width;

        let mut child_y = 0.0;
        for child in &mut layout_box.children {
            self.layout(child, final_width, text_renderer);
            child.dimensions.content.x = layout_box.dimensions.padding.left
                + layout_box.dimensions.border.left
                + child.dimensions.margin.left;
            child.dimensions.content.y = child_y
                + layout_box.dimensions.padding.top
                + layout_box.dimensions.border.top
                + child.dimensions.margin.top;

            child_y = child.dimensions.margin_box().bottom();
        }

        // Calculate content height based on box-sizing
        let content_height = match style.height {
            Some(specified_height) => {
                match style.box_sizing {
                    BoxSizing::ContentBox => specified_height,
                    BoxSizing::BorderBox => {
                        // border-box: specified height includes padding and border
                        (specified_height
                            - layout_box.dimensions.padding.vertical()
                            - layout_box.dimensions.border.vertical())
                        .max(0.0)
                    }
                }
            }
            None => child_y,
        };

        // Apply min/max height constraints
        let mut final_height = content_height;
        if let Some(min_height) = style.min_height {
            final_height = final_height.max(min_height);
        }
        if let Some(max_height) = style.max_height {
            final_height = final_height.min(max_height);
        }

        layout_box.dimensions.content.height = final_height;
    }

    fn layout_inline(&self, layout_box: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
        let mut total_width = 0.0;
        let mut max_height = 0.0f32;

        for child in &mut layout_box.children {
            self.layout(child, containing_width, text_renderer);
            child.dimensions.content.x = total_width;
            total_width += child.dimensions.margin_box().width;
            max_height = max_height.max(child.dimensions.margin_box().height);
        }

        layout_box.dimensions.content.width = total_width;
        layout_box.dimensions.content.height = max_height;
    }

    fn layout_text(&self, layout_box: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
        if let Some(text) = &layout_box.text_content {
            let font_size = layout_box.style.font_size;
            let (width, height) = text_renderer.measure_text_fast(text, font_size, containing_width);

            layout_box.dimensions.content.width = width;
            layout_box.dimensions.content.height = height;
        }
    }

    fn layout_image(&self, layout_box: &mut LayoutBox, containing_width: f32) {
        let style = &layout_box.style;

        // Set up margins, padding, borders
        layout_box.dimensions.margin = EdgeSizes::new(
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
        );
        layout_box.dimensions.padding = EdgeSizes::new(
            style.padding_top,
            style.padding_right,
            style.padding_bottom,
            style.padding_left,
        );
        layout_box.dimensions.border = EdgeSizes::new(
            style.border_top_width,
            style.border_right_width,
            style.border_bottom_width,
            style.border_left_width,
        );

        // Get intrinsic dimensions (from loaded image or HTML attributes)
        let (intrinsic_width, intrinsic_height) = layout_box
            .intrinsic_size
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((300.0, 150.0)); // Default placeholder size

        // Calculate available space
        let available_width = containing_width
            - layout_box.dimensions.margin.horizontal()
            - layout_box.dimensions.padding.horizontal()
            - layout_box.dimensions.border.horizontal();

        // Determine final dimensions based on CSS properties and intrinsic size
        let (final_width, final_height) = match (style.width, style.height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => {
                // Width specified, calculate height to maintain aspect ratio
                let aspect = if intrinsic_width > 0.0 {
                    intrinsic_height / intrinsic_width
                } else {
                    0.5
                };
                (w, w * aspect)
            }
            (None, Some(h)) => {
                // Height specified, calculate width to maintain aspect ratio
                let aspect = if intrinsic_height > 0.0 {
                    intrinsic_width / intrinsic_height
                } else {
                    2.0
                };
                (h * aspect, h)
            }
            (None, None) => {
                // Use intrinsic size, but constrain to available width
                if intrinsic_width > available_width && available_width > 0.0 {
                    let scale = available_width / intrinsic_width;
                    (available_width, intrinsic_height * scale)
                } else {
                    (intrinsic_width, intrinsic_height)
                }
            }
        };

        layout_box.dimensions.content.width = final_width;
        layout_box.dimensions.content.height = final_height;
    }

    pub fn build_display_list(&self, scroll_y: f32) -> DisplayList {
        let mut list = DisplayList::new();

        if let Some(root) = &self.root {
            self.render_layout_box(root, &mut list, 0.0, -scroll_y);
        }

        list
    }

    fn render_layout_box(&self, layout_box: &LayoutBox, list: &mut DisplayList, offset_x: f32, offset_y: f32) {
        let x = offset_x + layout_box.dimensions.content.x;
        let y = offset_y + layout_box.dimensions.content.y;
        let opacity = layout_box.style.opacity;
        let border_radius = layout_box.style.border_radius;

        // Render box shadow first (behind the element)
        if let Some(shadow) = &layout_box.style.box_shadow {
            let border_box = layout_box.dimensions.border_box();
            list.push_box_shadow(
                Rect::new(
                    offset_x + border_box.x,
                    offset_y + border_box.y,
                    border_box.width,
                    border_box.height,
                ),
                shadow.color,
                shadow.offset_x,
                shadow.offset_y,
                shadow.blur_radius,
                shadow.spread_radius,
                border_radius,
            );
        }

        if layout_box.style.background_color.a > 0.0 {
            let border_box = layout_box.dimensions.border_box();
            list.push_rect_with_radius(
                Rect::new(
                    offset_x + border_box.x,
                    offset_y + border_box.y,
                    border_box.width,
                    border_box.height,
                ),
                layout_box.style.background_color,
                border_radius,
                opacity,
            );
        }

        if layout_box.dimensions.border.top > 0.0
            || layout_box.dimensions.border.right > 0.0
            || layout_box.dimensions.border.bottom > 0.0
            || layout_box.dimensions.border.left > 0.0
        {
            let border_box = layout_box.dimensions.border_box();
            list.push_border_with_radius(
                Rect::new(
                    offset_x + border_box.x,
                    offset_y + border_box.y,
                    border_box.width,
                    border_box.height,
                ),
                layout_box.style.border_color,
                layout_box.dimensions.border.top,
                border_radius,
            );
        }

        if let Some(text) = &layout_box.text_content {
            list.push_text_with_opacity(
                text.clone(),
                x,
                y,
                layout_box.style.color,
                layout_box.style.font_size,
                opacity,
            );
        }

        // Render image if this is an image box with a loaded texture
        if layout_box.box_type == BoxType::Image {
            if let Some(texture_id) = layout_box.texture_id {
                let content_rect = Rect::new(
                    x,
                    y,
                    layout_box.dimensions.content.width,
                    layout_box.dimensions.content.height,
                );
                list.push_image_with_opacity(content_rect, texture_id, opacity);
            }
        }

        for child in &layout_box.children {
            self.render_layout_box(child, list, x, y);
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        if let Some(root) = &self.root {
            return self.hit_test_box(root, x, y + self.scroll_y, 0.0, 0.0);
        }
        None
    }

    fn hit_test_box(&self, layout_box: &LayoutBox, x: f32, y: f32, offset_x: f32, offset_y: f32) -> Option<NodeId> {
        let box_x = offset_x + layout_box.dimensions.content.x;
        let box_y = offset_y + layout_box.dimensions.content.y;

        let border_box = layout_box.dimensions.border_box();
        let rect = Rect::new(
            offset_x + border_box.x,
            offset_y + border_box.y,
            border_box.width,
            border_box.height,
        );

        if rect.contains(x, y) {
            for child in layout_box.children.iter().rev() {
                if let Some(node_id) = self.hit_test_box(child, x, y, box_x, box_y) {
                    return Some(node_id);
                }
            }

            return layout_box.node_id;
        }

        None
    }

    pub fn content_height(&self) -> f32 {
        self.root
            .as_ref()
            .map(|r| r.dimensions.margin_box().height)
            .unwrap_or(0.0)
    }

    pub fn scroll(&mut self, delta: f32) {
        let max_scroll = (self.content_height() - self.viewport_height).max(0.0);
        self.scroll_y = (self.scroll_y + delta).clamp(0.0, max_scroll);
    }

    // ====== Smooth Scrolling Methods ======

    pub fn update_scroll(&mut self, dt: f32) -> bool {
        let max_scroll = self.max_scroll();
        let animating = self.scroll_animator.update(dt, max_scroll);
        self.scroll_y = self.scroll_animator.current;
        animating
    }

    pub fn scroll_smooth(&mut self, delta: f32) {
        let max_scroll = self.max_scroll();
        self.scroll_animator.scroll_by(delta, max_scroll);
    }

    pub fn scroll_to(&mut self, position: f32) {
        let max_scroll = self.max_scroll();
        self.scroll_animator.scroll_to(position, max_scroll);
    }

    pub fn scroll_immediate(&mut self, position: f32) {
        let max_scroll = self.max_scroll();
        self.scroll_animator.scroll_immediate(position, max_scroll);
        self.scroll_y = self.scroll_animator.current;
    }

    pub fn is_scroll_animating(&self) -> bool {
        self.scroll_animator.animating
    }

    fn max_scroll(&self) -> f32 {
        (self.content_height() - self.viewport_height).max(0.0)
    }

    // ====== Scrollbar Methods ======

    pub fn needs_scrollbar(&self) -> bool {
        self.content_height() > self.viewport_height
    }

    pub fn scrollbar_track_rect(&self) -> Rect {
        Rect::new(
            self.viewport_width - self.scrollbar_config.width,
            0.0,
            self.scrollbar_config.width,
            self.viewport_height,
        )
    }

    pub fn scrollbar_thumb_rect(&self) -> Rect {
        let content_height = self.content_height();
        let viewport_height = self.viewport_height;

        if content_height <= viewport_height {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let visible_ratio = viewport_height / content_height;
        let thumb_height = (viewport_height * visible_ratio)
            .max(self.scrollbar_config.min_thumb_height);

        let max_scroll = content_height - viewport_height;
        let scroll_ratio = if max_scroll > 0.0 {
            self.scroll_y / max_scroll
        } else {
            0.0
        };

        let track_height = viewport_height;
        let thumb_travel = track_height - thumb_height;
        let thumb_y = scroll_ratio * thumb_travel;

        Rect::new(
            self.viewport_width - self.scrollbar_config.width,
            thumb_y,
            self.scrollbar_config.width,
            thumb_height,
        )
    }

    pub fn scrollbar_hit_test(&self, x: f32, y: f32) -> ScrollbarHitArea {
        if !self.needs_scrollbar() {
            return ScrollbarHitArea::None;
        }

        let track = self.scrollbar_track_rect();
        if !track.contains(x, y) {
            return ScrollbarHitArea::None;
        }

        let thumb = self.scrollbar_thumb_rect();
        if thumb.contains(x, y) {
            ScrollbarHitArea::Thumb
        } else {
            ScrollbarHitArea::Track
        }
    }

    pub fn track_y_to_scroll(&self, y: f32) -> f32 {
        let content_height = self.content_height();
        let viewport_height = self.viewport_height;

        if content_height <= viewport_height {
            return 0.0;
        }

        let thumb_height = self.scrollbar_thumb_rect().height;
        let track_height = viewport_height;
        let thumb_travel = track_height - thumb_height;

        if thumb_travel <= 0.0 {
            return 0.0;
        }

        let thumb_center_y = y - (thumb_height / 2.0);
        let scroll_ratio = (thumb_center_y / thumb_travel).clamp(0.0, 1.0);
        let max_scroll = content_height - viewport_height;

        scroll_ratio * max_scroll
    }

    pub fn begin_thumb_drag(&mut self, mouse_y: f32) {
        self.scrollbar_state = ScrollbarState::Dragging {
            start_y: mouse_y,
            start_scroll: self.scroll_y,
        };
    }

    pub fn update_thumb_drag(&mut self, mouse_y: f32) {
        if let ScrollbarState::Dragging { start_y, start_scroll } = self.scrollbar_state {
            let content_height = self.content_height();
            let viewport_height = self.viewport_height;

            if content_height <= viewport_height {
                return;
            }

            let thumb_height = self.scrollbar_thumb_rect().height;
            let thumb_travel = viewport_height - thumb_height;

            if thumb_travel <= 0.0 {
                return;
            }

            let max_scroll = content_height - viewport_height;
            let delta_y = mouse_y - start_y;
            let scroll_delta = (delta_y / thumb_travel) * max_scroll;

            self.scroll_immediate(start_scroll + scroll_delta);
        }
    }

    pub fn end_thumb_drag(&mut self) {
        self.scrollbar_state = ScrollbarState::Idle;
    }

    pub fn update_scrollbar_hover(&mut self, x: f32, y: f32) {
        if matches!(self.scrollbar_state, ScrollbarState::Dragging { .. }) {
            return;
        }

        let hit = self.scrollbar_hit_test(x, y);
        self.scrollbar_state = if hit == ScrollbarHitArea::Thumb || hit == ScrollbarHitArea::Track {
            ScrollbarState::Hovering
        } else {
            ScrollbarState::Idle
        };
    }

    pub fn render_scrollbar(&self, list: &mut DisplayList) {
        if !self.needs_scrollbar() {
            return;
        }

        let track = self.scrollbar_track_rect();
        list.push_rect(track, self.scrollbar_config.track_color);

        let thumb_color = match &self.scrollbar_state {
            ScrollbarState::Idle => self.scrollbar_config.thumb_color,
            ScrollbarState::Hovering => self.scrollbar_config.thumb_hover_color,
            ScrollbarState::Dragging { .. } => self.scrollbar_config.thumb_active_color,
        };

        let thumb = self.scrollbar_thumb_rect();
        list.push_rect(thumb, thumb_color);
    }

    pub fn is_dragging_scrollbar(&self) -> bool {
        matches!(self.scrollbar_state, ScrollbarState::Dragging { .. })
    }

    // ====== Image URL Methods ======

    /// Collect all image URLs from the layout tree
    pub fn collect_image_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();
        if let Some(root) = &self.root {
            Self::collect_image_urls_recursive(root, &mut urls);
        }
        urls
    }

    fn collect_image_urls_recursive(layout_box: &LayoutBox, urls: &mut Vec<String>) {
        if let Some(src) = &layout_box.image_src {
            if !urls.contains(src) {
                urls.push(src.clone());
            }
        }
        for child in &layout_box.children {
            Self::collect_image_urls_recursive(child, urls);
        }
    }

    /// Resolve image URLs in place and return list of resolved URLs
    /// This updates image_src in the layout boxes to resolved URLs
    pub fn resolve_image_urls(&mut self, base_url: Option<&url::Url>) -> Vec<String> {
        let mut urls = Vec::new();
        if let Some(root) = &mut self.root {
            Self::resolve_image_urls_recursive(root, base_url, &mut urls);
        }
        urls
    }

    fn resolve_image_urls_recursive(
        layout_box: &mut LayoutBox,
        base_url: Option<&url::Url>,
        urls: &mut Vec<String>,
    ) {
        if let Some(src) = &layout_box.image_src {
            // Resolve the URL
            let resolved = if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
                Some(src.clone())
            } else if let Some(base) = base_url {
                base.join(src).ok().map(|u| u.to_string())
            } else {
                None
            };

            if let Some(resolved_url) = resolved {
                // Update the image_src to the resolved URL
                layout_box.image_src = Some(resolved_url.clone());
                if !urls.contains(&resolved_url) {
                    urls.push(resolved_url);
                }
            }
        }
        for child in &mut layout_box.children {
            Self::resolve_image_urls_recursive(child, base_url, urls);
        }
    }

    /// Update texture IDs for images with a given URL
    pub fn update_image_texture(&mut self, url: &str, texture_id: usize, size: ImageSize) {
        if let Some(root) = &mut self.root {
            Self::update_image_texture_recursive(root, url, texture_id, size);
        }
    }

    fn update_image_texture_recursive(
        layout_box: &mut LayoutBox,
        url: &str,
        texture_id: usize,
        size: ImageSize,
    ) {
        if let Some(src) = &layout_box.image_src {
            if src == url {
                layout_box.texture_id = Some(texture_id);
                // Update intrinsic size if not already set from attributes
                if layout_box.intrinsic_size.is_none() {
                    layout_box.intrinsic_size = Some(size);
                }
            }
        }
        for child in &mut layout_box.children {
            Self::update_image_texture_recursive(child, url, texture_id, size);
        }
    }
}
