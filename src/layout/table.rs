//! Table layout algorithm.
//!
//! Implements basic CSS table layout for rendering HTML tables.
//! This handles <table>, <tr>, <td>, <th>, <thead>, <tbody>, <tfoot> elements.

use super::box_model::EdgeSizes;
use super::tree::{BoxType, LayoutBox};
use crate::render::text::TextRenderer;

/// Layout a table element and all its children.
pub fn layout_table(layout_box: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
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

    let border_spacing = style.border_spacing;
    let border_collapse = style.border_collapse;

    // Calculate available content width
    let available_width = style.width.map(|w| w.to_px(containing_width)).unwrap_or_else(|| {
        containing_width
            - layout_box.dimensions.margin.horizontal()
            - layout_box.dimensions.padding.horizontal()
            - layout_box.dimensions.border.horizontal()
    });

    // Collect all rows (may be wrapped in row groups)
    let rows = collect_table_rows(layout_box);
    if rows.is_empty() {
        layout_box.dimensions.content.width = available_width;
        layout_box.dimensions.content.height = 0.0;
        return;
    }

    // Calculate column count by finding max cells in any row
    let num_columns = calculate_column_count(&rows, &layout_box.children);

    if num_columns == 0 {
        layout_box.dimensions.content.width = available_width;
        layout_box.dimensions.content.height = 0.0;
        return;
    }

    // Calculate column widths
    // For now, use equal width distribution. A more sophisticated algorithm
    // would consider intrinsic widths and CSS width properties.
    let spacing_width = if border_collapse {
        0.0
    } else {
        border_spacing * (num_columns as f32 + 1.0)
    };
    let column_width = (available_width - spacing_width) / num_columns as f32;

    // Layout rows and cells
    let mut y_position = if border_collapse { 0.0 } else { border_spacing };

    for child in &mut layout_box.children {
        match child.box_type {
            BoxType::TableRow => {
                let row_height = layout_table_row(
                    child,
                    column_width,
                    num_columns,
                    border_spacing,
                    border_collapse,
                    text_renderer,
                );
                child.dimensions.content.x = layout_box.dimensions.padding.left
                    + layout_box.dimensions.border.left;
                child.dimensions.content.y = y_position
                    + layout_box.dimensions.padding.top
                    + layout_box.dimensions.border.top;
                y_position += row_height + if border_collapse { 0.0 } else { border_spacing };
            }
            BoxType::TableRowGroup => {
                // Layout rows within the row group
                let mut group_y = 0.0;
                for row in &mut child.children {
                    if row.box_type == BoxType::TableRow {
                        let row_height = layout_table_row(
                            row,
                            column_width,
                            num_columns,
                            border_spacing,
                            border_collapse,
                            text_renderer,
                        );
                        row.dimensions.content.x = 0.0;
                        row.dimensions.content.y = group_y;
                        group_y += row_height + if border_collapse { 0.0 } else { border_spacing };
                    }
                }
                child.dimensions.content.width = available_width;
                child.dimensions.content.height = group_y;
                child.dimensions.content.x = layout_box.dimensions.padding.left
                    + layout_box.dimensions.border.left;
                child.dimensions.content.y = y_position
                    + layout_box.dimensions.padding.top
                    + layout_box.dimensions.border.top;
                y_position += group_y + if border_collapse { 0.0 } else { border_spacing };
            }
            _ => {
                // Caption or other elements - treat as block
                layout_block_child(child, available_width, text_renderer);
                child.dimensions.content.x = layout_box.dimensions.padding.left
                    + layout_box.dimensions.border.left
                    + child.dimensions.margin.left;
                child.dimensions.content.y = y_position
                    + layout_box.dimensions.padding.top
                    + layout_box.dimensions.border.top
                    + child.dimensions.margin.top;
                y_position += child.dimensions.margin_box().height;
            }
        }
    }

    layout_box.dimensions.content.width = available_width;
    layout_box.dimensions.content.height = style.height.map(|h| h.to_px(y_position)).unwrap_or(y_position);
}

/// Collect row indices from table children (handles both direct rows and row groups)
fn collect_table_rows(layout_box: &LayoutBox) -> Vec<usize> {
    let mut rows = Vec::new();
    for (i, child) in layout_box.children.iter().enumerate() {
        match child.box_type {
            BoxType::TableRow => rows.push(i),
            BoxType::TableRowGroup => {
                // Include indices for nested rows
                for _ in 0..child.children.len() {
                    rows.push(i);
                }
            }
            _ => {}
        }
    }
    rows
}

/// Calculate the number of columns by finding the maximum cells in any row
fn calculate_column_count(_rows: &[usize], children: &[LayoutBox]) -> usize {
    let mut max_columns = 0;

    for child in children {
        match child.box_type {
            BoxType::TableRow => {
                let cell_count = child.children.iter()
                    .filter(|c| c.box_type == BoxType::TableCell)
                    .count();
                max_columns = max_columns.max(cell_count);
            }
            BoxType::TableRowGroup => {
                for row in &child.children {
                    if row.box_type == BoxType::TableRow {
                        let cell_count = row.children.iter()
                            .filter(|c| c.box_type == BoxType::TableCell)
                            .count();
                        max_columns = max_columns.max(cell_count);
                    }
                }
            }
            _ => {}
        }
    }

    max_columns
}

/// Layout a single table row and return its height
fn layout_table_row(
    row: &mut LayoutBox,
    column_width: f32,
    num_columns: usize,
    border_spacing: f32,
    border_collapse: bool,
    text_renderer: &mut TextRenderer,
) -> f32 {
    let style = &row.style;

    // Set up row's box model
    row.dimensions.margin = EdgeSizes::new(
        style.margin_top,
        style.margin_right,
        style.margin_bottom,
        style.margin_left,
    );
    row.dimensions.padding = EdgeSizes::new(
        style.padding_top,
        style.padding_right,
        style.padding_bottom,
        style.padding_left,
    );
    row.dimensions.border = EdgeSizes::new(
        style.border_top_width,
        style.border_right_width,
        style.border_bottom_width,
        style.border_left_width,
    );

    let mut x_position = if border_collapse { 0.0 } else { border_spacing };
    let mut max_height: f32 = 0.0;

    // Layout each cell
    for child in &mut row.children {
        if child.box_type == BoxType::TableCell {
            layout_table_cell(child, column_width, text_renderer);
            child.dimensions.content.x = x_position + child.dimensions.margin.left
                + child.dimensions.border.left + child.dimensions.padding.left;
            child.dimensions.content.y = child.dimensions.margin.top
                + child.dimensions.border.top + child.dimensions.padding.top;

            x_position += column_width + if border_collapse { 0.0 } else { border_spacing };
            max_height = max_height.max(child.dimensions.margin_box().height);
        }
    }

    // Set row dimensions
    let row_width = if border_collapse {
        column_width * num_columns as f32
    } else {
        border_spacing + (column_width + border_spacing) * num_columns as f32
    };

    row.dimensions.content.width = row_width;
    row.dimensions.content.height = max_height;

    max_height
}

/// Layout a single table cell
fn layout_table_cell(cell: &mut LayoutBox, available_width: f32, text_renderer: &mut TextRenderer) {
    let style = &cell.style;

    cell.dimensions.margin = EdgeSizes::new(
        style.margin_top,
        style.margin_right,
        style.margin_bottom,
        style.margin_left,
    );
    cell.dimensions.padding = EdgeSizes::new(
        style.padding_top,
        style.padding_right,
        style.padding_bottom,
        style.padding_left,
    );
    cell.dimensions.border = EdgeSizes::new(
        style.border_top_width,
        style.border_right_width,
        style.border_bottom_width,
        style.border_left_width,
    );

    let content_width = available_width
        - cell.dimensions.margin.horizontal()
        - cell.dimensions.padding.horizontal()
        - cell.dimensions.border.horizontal();

    cell.dimensions.content.width = content_width;

    // Layout cell contents
    let mut child_y = 0.0;
    for child in &mut cell.children {
        layout_block_child(child, content_width, text_renderer);
        child.dimensions.content.x = child.dimensions.margin.left;
        child.dimensions.content.y = child_y + child.dimensions.margin.top;
        child_y = child.dimensions.margin_box().bottom();
    }

    cell.dimensions.content.height = style.height.map(|h| h.to_px(child_y)).unwrap_or(child_y);
}

/// Layout a block-level child (fallback for non-table elements within tables)
fn layout_block_child(child: &mut LayoutBox, containing_width: f32, text_renderer: &mut TextRenderer) {
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

    let content_width = style.width.map(|w| w.to_px(containing_width)).unwrap_or_else(|| {
        containing_width
            - child.dimensions.margin.horizontal()
            - child.dimensions.padding.horizontal()
            - child.dimensions.border.horizontal()
    });

    child.dimensions.content.width = content_width;

    // Handle text content
    if let Some(ref text) = child.text_content {
        let (_, height) = text_renderer.measure_text_fast(text, child.style.font_size, content_width);
        child.dimensions.content.height = height;
    } else {
        // Layout nested children
        let mut child_y = 0.0;
        for nested in &mut child.children {
            layout_block_child(nested, content_width, text_renderer);
            nested.dimensions.content.x = nested.dimensions.margin.left;
            nested.dimensions.content.y = child_y + nested.dimensions.margin.top;
            child_y = nested.dimensions.margin_box().bottom();
        }
        child.dimensions.content.height = style.height.map(|h| h.to_px(child_y)).unwrap_or(child_y);
    }
}
