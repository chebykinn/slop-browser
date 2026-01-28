//! Flexbox layout algorithm implementation
//!
//! Implements CSS Flexbox Layout Module Level 1
//! https://www.w3.org/TR/css-flexbox-1/

use super::box_model::EdgeSizes;
use super::tree::{BoxType, LayoutBox};
use crate::css::computed::{AlignItems, FlexDirection, FlexWrap, JustifyContent};
use crate::render::text::TextRenderer;

/// A flex item with computed sizing info
struct FlexItem<'a> {
    layout_box: &'a mut LayoutBox,
    /// Outer flex basis (including margin/border/padding)
    outer_flex_basis: f32,
    /// Whether this item can grow
    flex_grow: f32,
    /// Whether this item can shrink
    flex_shrink: f32,
    /// The item's hypothetical main size
    hypothetical_main_size: f32,
    /// Main axis position offset
    main_offset: f32,
    /// Cross axis position offset
    cross_offset: f32,
    /// Resolved main size
    main_size: f32,
    /// Resolved cross size
    cross_size: f32,
}

/// A flex line containing multiple flex items
struct FlexLine<'a> {
    items: Vec<FlexItem<'a>>,
    /// Total main size of items in this line
    main_size: f32,
    /// Cross size of this line (max cross size of items)
    cross_size: f32,
}

/// Perform flex layout on a flex container
pub fn layout_flex(
    layout_box: &mut LayoutBox,
    containing_width: f32,
    text_renderer: &mut TextRenderer,
) {
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

    // Calculate container's content width
    let content_width = style.width.unwrap_or_else(|| {
        containing_width
            - layout_box.dimensions.margin.horizontal()
            - layout_box.dimensions.padding.horizontal()
            - layout_box.dimensions.border.horizontal()
    });

    layout_box.dimensions.content.width = content_width;

    // Get flex properties
    let flex_direction = style.flex_direction;
    let flex_wrap = style.flex_wrap;
    let justify_content = style.justify_content;
    let align_items = style.align_items;
    let gap = style.gap;
    let row_gap = if style.row_gap > 0.0 { style.row_gap } else { gap };
    let column_gap = if style.column_gap > 0.0 { style.column_gap } else { gap };

    let is_row = matches!(flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
    let is_reversed = matches!(
        flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );

    let main_gap = if is_row { column_gap } else { row_gap };
    let cross_gap = if is_row { row_gap } else { column_gap };

    // Available main size
    let available_main = if is_row { content_width } else { f32::MAX };

    // First pass: layout children to get their intrinsic sizes
    for child in &mut layout_box.children {
        layout_child_intrinsic(child, content_width, text_renderer);
    }

    // Collect items and sort by order
    let mut items: Vec<_> = layout_box
        .children
        .iter_mut()
        .filter(|c| c.box_type != BoxType::Text || c.text_content.is_some())
        .collect();

    // Sort by CSS order property
    items.sort_by_key(|item| item.style.order);

    // Build flex lines
    let mut lines: Vec<FlexLine> = Vec::new();
    let mut current_line_items: Vec<FlexItem> = Vec::new();
    let mut current_line_main_size = 0.0f32;

    for item in items {
        let flex_basis = item.style.flex_basis.unwrap_or_else(|| {
            if is_row {
                item.dimensions.margin_box().width
            } else {
                item.dimensions.margin_box().height
            }
        });

        let outer_main = if is_row {
            flex_basis
                + item.dimensions.margin.horizontal()
                + item.dimensions.padding.horizontal()
                + item.dimensions.border.horizontal()
        } else {
            flex_basis
                + item.dimensions.margin.vertical()
                + item.dimensions.padding.vertical()
                + item.dimensions.border.vertical()
        };

        // Check if we need to wrap
        let gap_for_this_item = if current_line_items.is_empty() { 0.0 } else { main_gap };
        let would_overflow = flex_wrap != FlexWrap::NoWrap
            && !current_line_items.is_empty()
            && current_line_main_size + gap_for_this_item + outer_main > available_main;

        if would_overflow {
            // Finish current line
            let line = build_flex_line(current_line_items, current_line_main_size, is_row);
            lines.push(line);
            current_line_items = Vec::new();
            current_line_main_size = 0.0;
        }

        if !current_line_items.is_empty() {
            current_line_main_size += main_gap;
        }

        let hypothetical_main = flex_basis;
        let flex_grow = item.style.flex_grow;
        let flex_shrink = item.style.flex_shrink;

        current_line_items.push(FlexItem {
            layout_box: item,
            outer_flex_basis: outer_main,
            flex_grow,
            flex_shrink,
            hypothetical_main_size: hypothetical_main,
            main_offset: 0.0,
            cross_offset: 0.0,
            main_size: hypothetical_main,
            cross_size: 0.0,
        });

        current_line_main_size += outer_main;
    }

    // Don't forget the last line
    if !current_line_items.is_empty() {
        let line = build_flex_line(current_line_items, current_line_main_size, is_row);
        lines.push(line);
    }

    // Resolve flexible lengths for each line
    for line in &mut lines {
        resolve_flexible_lengths(line, available_main, main_gap, is_row);
    }

    // Position items on main axis (justify-content)
    for line in &mut lines {
        position_main_axis(line, available_main, justify_content, main_gap, is_reversed);
    }

    // Calculate line cross sizes and position items on cross axis (align-items)
    for line in &mut lines {
        calculate_line_cross_size(line, is_row);
        position_cross_axis_in_line(line, align_items, is_row);
    }

    // Position lines on cross axis
    let total_cross_size = position_lines(
        &mut lines,
        cross_gap,
        flex_wrap == FlexWrap::WrapReverse,
    );

    // Apply final positions to layout boxes
    apply_positions(&mut lines, is_row, &layout_box.dimensions.padding, &layout_box.dimensions.border);

    // Set container height
    let container_height = style.height.unwrap_or(total_cross_size);
    layout_box.dimensions.content.height = container_height;
}

/// Layout a child to determine its intrinsic size
fn layout_child_intrinsic(child: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
    let style = &child.style;

    child.dimensions.margin = EdgeSizes::new(
        style.margin_top,
        style.margin_right,
        style.margin_bottom,
        style.margin_left,
    );
    child.dimensions.padding = EdgeSizes::new(
        style.padding_top,
        style.padding_right,
        style.padding_bottom,
        style.padding_left,
    );
    child.dimensions.border = EdgeSizes::new(
        style.border_top_width,
        style.border_right_width,
        style.border_bottom_width,
        style.border_left_width,
    );

    // Calculate content size based on type
    if let Some(text) = &child.text_content {
        let (width, height) = text_renderer.measure_text(text, style.font_size);
        child.dimensions.content.width = width;
        child.dimensions.content.height = height;
    } else if let Some(w) = style.width {
        child.dimensions.content.width = w;
    } else {
        // Use available width minus margins/padding/border for block-level
        let available = containing_width
            - child.dimensions.margin.horizontal()
            - child.dimensions.padding.horizontal()
            - child.dimensions.border.horizontal();
        child.dimensions.content.width = available.max(0.0);
    }

    if let Some(h) = style.height {
        child.dimensions.content.height = h;
    } else if child.text_content.is_none() {
        // For non-text elements without explicit height, calculate from children
        let mut child_height = 0.0f32;
        for grandchild in &mut child.children {
            layout_child_intrinsic(grandchild, child.dimensions.content.width, text_renderer);
            child_height += grandchild.dimensions.margin_box().height;
        }
        child.dimensions.content.height = child_height;
    }
}

/// Build a flex line from collected items
fn build_flex_line(items: Vec<FlexItem>, main_size: f32, is_row: bool) -> FlexLine {
    let cross_size = items
        .iter()
        .map(|item| {
            if is_row {
                item.layout_box.dimensions.margin_box().height
            } else {
                item.layout_box.dimensions.margin_box().width
            }
        })
        .fold(0.0f32, |a, b| a.max(b));

    FlexLine {
        items,
        main_size,
        cross_size,
    }
}

/// Resolve flexible lengths (grow/shrink)
fn resolve_flexible_lengths(line: &mut FlexLine, available_main: f32, main_gap: f32, is_row: bool) {
    if line.items.is_empty() {
        return;
    }

    let total_gaps = main_gap * (line.items.len() - 1) as f32;
    let free_space = available_main - line.main_size;

    if free_space > 0.0 {
        // Grow
        let total_grow: f32 = line.items.iter().map(|i| i.flex_grow).sum();
        if total_grow > 0.0 {
            let grow_per_unit = free_space / total_grow;
            for item in &mut line.items {
                let grow = item.flex_grow * grow_per_unit;
                item.main_size = item.hypothetical_main_size + grow;
            }
        }
    } else if free_space < 0.0 {
        // Shrink
        let total_shrink: f32 = line.items.iter().map(|i| i.flex_shrink * i.hypothetical_main_size).sum();
        if total_shrink > 0.0 {
            let shrink_ratio = (-free_space) / total_shrink;
            for item in &mut line.items {
                let shrink = item.flex_shrink * item.hypothetical_main_size * shrink_ratio;
                item.main_size = (item.hypothetical_main_size - shrink).max(0.0);
            }
        }
    }

    // Update layout box dimensions
    for item in &mut line.items {
        if is_row {
            item.layout_box.dimensions.content.width = item.main_size;
        } else {
            item.layout_box.dimensions.content.height = item.main_size;
        }
    }

    // Recalculate line main size
    line.main_size = line
        .items
        .iter()
        .map(|i| {
            if is_row {
                i.layout_box.dimensions.margin_box().width
            } else {
                i.layout_box.dimensions.margin_box().height
            }
        })
        .sum::<f32>()
        + total_gaps;
}

/// Position items along main axis
fn position_main_axis(
    line: &mut FlexLine,
    available_main: f32,
    justify_content: JustifyContent,
    main_gap: f32,
    is_reversed: bool,
) {
    if line.items.is_empty() {
        return;
    }

    let free_space = (available_main - line.main_size).max(0.0);
    let item_count = line.items.len();

    let (initial_offset, between_space) = match justify_content {
        JustifyContent::FlexStart => (0.0, main_gap),
        JustifyContent::FlexEnd => (free_space, main_gap),
        JustifyContent::Center => (free_space / 2.0, main_gap),
        JustifyContent::SpaceBetween => {
            if item_count > 1 {
                (0.0, free_space / (item_count - 1) as f32 + main_gap)
            } else {
                (0.0, main_gap)
            }
        }
        JustifyContent::SpaceAround => {
            let space = free_space / item_count as f32;
            (space / 2.0, space + main_gap)
        }
        JustifyContent::SpaceEvenly => {
            let space = free_space / (item_count + 1) as f32;
            (space, space + main_gap)
        }
    };

    let mut offset = initial_offset;
    let items: Vec<_> = if is_reversed {
        line.items.iter_mut().rev().collect()
    } else {
        line.items.iter_mut().collect()
    };

    for (i, item) in items.into_iter().enumerate() {
        item.main_offset = offset;
        let item_main = item.layout_box.dimensions.margin_box().width.max(
            item.layout_box.dimensions.margin_box().height,
        );
        offset += item_main;
        if i < item_count - 1 {
            offset += between_space;
        }
    }
}

/// Calculate the cross size of a flex line
fn calculate_line_cross_size(line: &mut FlexLine, is_row: bool) {
    line.cross_size = line
        .items
        .iter()
        .map(|item| {
            if is_row {
                item.layout_box.dimensions.margin_box().height
            } else {
                item.layout_box.dimensions.margin_box().width
            }
        })
        .fold(0.0f32, |a, b| a.max(b));
}

/// Position items within a line on the cross axis
fn position_cross_axis_in_line(line: &mut FlexLine, align_items: AlignItems, is_row: bool) {
    for item in &mut line.items {
        let item_cross = if is_row {
            item.layout_box.dimensions.margin_box().height
        } else {
            item.layout_box.dimensions.margin_box().width
        };

        // Check for align-self override
        let align = item.layout_box.style.align_self.unwrap_or(align_items);

        item.cross_offset = match align {
            AlignItems::FlexStart => 0.0,
            AlignItems::FlexEnd => line.cross_size - item_cross,
            AlignItems::Center => (line.cross_size - item_cross) / 2.0,
            AlignItems::Stretch => {
                // Stretch the item to fill cross size
                if is_row {
                    item.layout_box.dimensions.content.height = line.cross_size
                        - item.layout_box.dimensions.margin.vertical()
                        - item.layout_box.dimensions.padding.vertical()
                        - item.layout_box.dimensions.border.vertical();
                } else {
                    item.layout_box.dimensions.content.width = line.cross_size
                        - item.layout_box.dimensions.margin.horizontal()
                        - item.layout_box.dimensions.padding.horizontal()
                        - item.layout_box.dimensions.border.horizontal();
                }
                0.0
            }
            AlignItems::Baseline => 0.0, // Simplified: treat as flex-start
        };

        item.cross_size = line.cross_size;
    }
}

/// Position lines and return total cross size
fn position_lines(lines: &mut [FlexLine], cross_gap: f32, wrap_reverse: bool) -> f32 {
    let mut cross_offset = 0.0f32;
    let line_count = lines.len();

    let lines_iter: Vec<_> = if wrap_reverse {
        lines.iter_mut().rev().collect()
    } else {
        lines.iter_mut().collect()
    };

    for (i, line) in lines_iter.into_iter().enumerate() {
        for item in &mut line.items {
            item.cross_offset += cross_offset;
        }
        cross_offset += line.cross_size;
        if i < line_count - 1 {
            cross_offset += cross_gap;
        }
    }

    cross_offset
}

/// Apply computed positions to layout boxes
fn apply_positions(lines: &mut [FlexLine], is_row: bool, padding: &EdgeSizes, border: &EdgeSizes) {
    for line in lines {
        for item in &mut line.items {
            let margin = &item.layout_box.dimensions.margin;

            if is_row {
                item.layout_box.dimensions.content.x = item.main_offset
                    + margin.left
                    + padding.left
                    + border.left;
                item.layout_box.dimensions.content.y = item.cross_offset
                    + margin.top
                    + padding.top
                    + border.top;
            } else {
                item.layout_box.dimensions.content.x = item.cross_offset
                    + margin.left
                    + padding.left
                    + border.left;
                item.layout_box.dimensions.content.y = item.main_offset
                    + margin.top
                    + padding.top
                    + border.top;
            }
        }
    }
}

// Note: Unit tests removed because TextRenderer requires GPU context.
// Flexbox functionality is tested via integration tests.
