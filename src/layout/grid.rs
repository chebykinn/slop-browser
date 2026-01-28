//! CSS Grid layout algorithm implementation
//!
//! Implements a simplified version of CSS Grid Layout Module Level 1
//! https://www.w3.org/TR/css-grid-1/

use super::box_model::EdgeSizes;
use super::tree::{BoxType, LayoutBox};
use crate::css::computed::GridTrackSize;
use crate::render::text::TextRenderer;

/// A grid item with placement information
struct GridItem<'a> {
    layout_box: &'a mut LayoutBox,
    column_start: i32,
    column_end: i32,
    row_start: i32,
    #[allow(dead_code)] // Used for row spanning in future
    row_end: i32,
}

/// Resolved track sizes
struct ResolvedTracks {
    sizes: Vec<f32>,
    positions: Vec<f32>,
}

/// Perform grid layout on a grid container
pub fn layout_grid(
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
    let content_width = style.width.map(|w| w.to_px(containing_width)).unwrap_or_else(|| {
        containing_width
            - layout_box.dimensions.margin.horizontal()
            - layout_box.dimensions.padding.horizontal()
            - layout_box.dimensions.border.horizontal()
    });

    layout_box.dimensions.content.width = content_width;

    // Get grid properties
    let column_templates = &style.grid_template_columns;
    let row_templates = &style.grid_template_rows;
    let column_gap = if style.grid_column_gap > 0.0 {
        style.grid_column_gap
    } else {
        style.grid_gap
    };
    let row_gap = if style.grid_row_gap > 0.0 {
        style.grid_row_gap
    } else {
        style.grid_gap
    };

    // First pass: layout children to get their intrinsic sizes
    for child in &mut layout_box.children {
        layout_child_intrinsic(child, content_width, text_renderer);
    }

    // Determine grid dimensions
    let (num_columns, num_rows) = determine_grid_size(
        &layout_box.children,
        column_templates.len(),
        row_templates.len(),
    );

    // Resolve column tracks
    let column_tracks = resolve_tracks(
        column_templates,
        num_columns,
        content_width,
        column_gap,
        &style.grid_auto_columns,
    );

    // Collect items with their placement
    let mut items: Vec<GridItem> = layout_box
        .children
        .iter_mut()
        .filter(|c| c.box_type != BoxType::Text || c.text_content.is_some())
        .enumerate()
        .map(|(i, child)| {
            let placement = get_item_placement(child, i, num_columns);
            GridItem {
                layout_box: child,
                column_start: placement.0,
                column_end: placement.1,
                row_start: placement.2,
                row_end: placement.3,
            }
        })
        .collect();

    // Calculate row sizes based on item content
    let row_heights = calculate_row_heights(&mut items, &column_tracks, num_rows, row_templates, text_renderer);
    let row_tracks = resolve_row_tracks(row_heights, row_gap);

    // Position items
    for item in &mut items {
        position_grid_item(
            item,
            &column_tracks,
            &row_tracks,
            &layout_box.dimensions.padding,
            &layout_box.dimensions.border,
        );
    }

    // Set container height
    let total_height = row_tracks.positions.last().copied().unwrap_or(0.0)
        + row_tracks.sizes.last().copied().unwrap_or(0.0);
    layout_box.dimensions.content.height = style.height.map(|h| h.to_px(total_height)).unwrap_or(total_height);
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
        child.dimensions.content.width = w.to_px(containing_width);
    } else {
        let available = containing_width
            - child.dimensions.margin.horizontal()
            - child.dimensions.padding.horizontal()
            - child.dimensions.border.horizontal();
        child.dimensions.content.width = available.max(0.0);
    }

    if let Some(h) = style.height {
        child.dimensions.content.height = h.to_px(child.dimensions.content.height);
    } else if child.text_content.is_none() {
        let mut child_height = 0.0f32;
        for grandchild in &mut child.children {
            layout_child_intrinsic(grandchild, child.dimensions.content.width, text_renderer);
            child_height += grandchild.dimensions.margin_box().height;
        }
        child.dimensions.content.height = child_height;
    }
}

/// Determine the size of the grid (number of columns and rows)
fn determine_grid_size(
    children: &[LayoutBox],
    template_columns: usize,
    template_rows: usize,
) -> (usize, usize) {
    let mut max_column = template_columns.max(1);
    let mut max_row = template_rows.max(1);

    for (i, child) in children.iter().enumerate() {
        let placement = &child.style.grid_column;
        if let Some(end) = placement.end {
            max_column = max_column.max(end as usize);
        } else if let Some(start) = placement.start {
            max_column = max_column.max(start as usize);
        }

        let row_placement = &child.style.grid_row;
        if let Some(end) = row_placement.end {
            max_row = max_row.max(end as usize);
        } else if let Some(start) = row_placement.start {
            max_row = max_row.max(start as usize);
        }

        // For auto-placed items, calculate based on item index
        if placement.start.is_none() && template_columns > 0 {
            let auto_row = (i / template_columns) + 1;
            max_row = max_row.max(auto_row);
        }
    }

    // Ensure at least enough rows for all children
    if template_columns > 0 {
        let needed_rows = (children.len() + template_columns - 1) / template_columns;
        max_row = max_row.max(needed_rows);
    }

    (max_column, max_row)
}

/// Get item placement (column_start, column_end, row_start, row_end)
fn get_item_placement(child: &LayoutBox, index: usize, num_columns: usize) -> (i32, i32, i32, i32) {
    let col_placement = &child.style.grid_column;
    let row_placement = &child.style.grid_row;

    let (col_start, col_end) = if let Some(start) = col_placement.start {
        let end = col_placement.end.unwrap_or(start + 1);
        (start, end)
    } else {
        // Auto placement
        let col = (index % num_columns.max(1)) as i32 + 1;
        (col, col + 1)
    };

    let (row_start, row_end) = if let Some(start) = row_placement.start {
        let end = row_placement.end.unwrap_or(start + 1);
        (start, end)
    } else {
        // Auto placement
        let row = (index / num_columns.max(1)) as i32 + 1;
        (row, row + 1)
    };

    (col_start, col_end, row_start, row_end)
}

/// Resolve track sizes for columns
fn resolve_tracks(
    templates: &[GridTrackSize],
    num_tracks: usize,
    available_space: f32,
    gap: f32,
    auto_size: &GridTrackSize,
) -> ResolvedTracks {
    let actual_num_tracks = num_tracks.max(templates.len()).max(1);
    let total_gaps = gap * (actual_num_tracks.saturating_sub(1)) as f32;
    let space_for_tracks = (available_space - total_gaps).max(0.0);

    let mut sizes = Vec::with_capacity(actual_num_tracks);
    let mut total_fr = 0.0f32;
    let mut fixed_size = 0.0f32;

    // First pass: calculate fixed sizes and total fr units
    for i in 0..actual_num_tracks {
        let template = templates.get(i).unwrap_or(auto_size);
        match template {
            GridTrackSize::Px(px) => {
                sizes.push(*px);
                fixed_size += *px;
            }
            GridTrackSize::Fr(fr) => {
                sizes.push(*fr); // Temporarily store fr value
                total_fr += *fr;
            }
            GridTrackSize::Auto | GridTrackSize::MinContent | GridTrackSize::MaxContent => {
                // Use equal distribution for auto tracks
                sizes.push(0.0);
                total_fr += 1.0; // Treat auto as 1fr
            }
        }
    }

    // Second pass: resolve fr units
    let space_for_fr = (space_for_tracks - fixed_size).max(0.0);
    let fr_unit_size = if total_fr > 0.0 {
        space_for_fr / total_fr
    } else {
        0.0
    };

    for (i, size) in sizes.iter_mut().enumerate() {
        let template = templates.get(i).unwrap_or(auto_size);
        match template {
            GridTrackSize::Px(_) => {}
            GridTrackSize::Fr(fr) => {
                *size = fr * fr_unit_size;
            }
            GridTrackSize::Auto | GridTrackSize::MinContent | GridTrackSize::MaxContent => {
                *size = fr_unit_size; // 1fr equivalent
            }
        }
    }

    // Calculate positions
    let mut positions = Vec::with_capacity(actual_num_tracks);
    let mut pos = 0.0;
    for (i, &size) in sizes.iter().enumerate() {
        positions.push(pos);
        pos += size;
        if i < actual_num_tracks - 1 {
            pos += gap;
        }
    }

    ResolvedTracks { sizes, positions }
}

/// Calculate row heights based on item content
fn calculate_row_heights(
    items: &mut [GridItem],
    column_tracks: &ResolvedTracks,
    num_rows: usize,
    row_templates: &[GridTrackSize],
    _text_renderer: &mut TextRenderer,
) -> Vec<f32> {
    let mut heights = vec![0.0f32; num_rows];

    for item in items {
        let row_index = (item.row_start - 1) as usize;
        if row_index >= num_rows {
            continue;
        }

        // Calculate the width available to this item
        let col_start = (item.column_start - 1) as usize;
        let col_end = (item.column_end - 1) as usize;

        let item_width: f32 = (col_start..col_end.min(column_tracks.sizes.len()))
            .map(|i| column_tracks.sizes[i])
            .sum();

        // Update item width if it spans columns
        if item_width > 0.0 {
            item.layout_box.dimensions.content.width = item_width
                - item.layout_box.dimensions.margin.horizontal()
                - item.layout_box.dimensions.padding.horizontal()
                - item.layout_box.dimensions.border.horizontal();
        }

        // Get the item's height
        let item_height = item.layout_box.dimensions.margin_box().height;

        // Check if there's a template for this row
        if let Some(template) = row_templates.get(row_index) {
            match template {
                GridTrackSize::Px(px) => {
                    heights[row_index] = heights[row_index].max(*px);
                    continue;
                }
                _ => {}
            }
        }

        heights[row_index] = heights[row_index].max(item_height);
    }

    // Ensure minimum heights for empty rows
    for height in &mut heights {
        if *height == 0.0 {
            *height = 20.0; // Minimum row height
        }
    }

    heights
}

/// Resolve row track positions from heights
fn resolve_row_tracks(heights: Vec<f32>, gap: f32) -> ResolvedTracks {
    let mut positions = Vec::with_capacity(heights.len());
    let mut pos = 0.0;

    for (i, &height) in heights.iter().enumerate() {
        positions.push(pos);
        pos += height;
        if i < heights.len() - 1 {
            pos += gap;
        }
    }

    ResolvedTracks {
        sizes: heights,
        positions,
    }
}

/// Position a grid item within the grid
fn position_grid_item(
    item: &mut GridItem,
    column_tracks: &ResolvedTracks,
    row_tracks: &ResolvedTracks,
    padding: &EdgeSizes,
    border: &EdgeSizes,
) {
    let col_start = (item.column_start - 1) as usize;
    let row_start = (item.row_start - 1) as usize;

    let x = column_tracks.positions.get(col_start).copied().unwrap_or(0.0);
    let y = row_tracks.positions.get(row_start).copied().unwrap_or(0.0);

    item.layout_box.dimensions.content.x = x
        + item.layout_box.dimensions.margin.left
        + padding.left
        + border.left;
    item.layout_box.dimensions.content.y = y
        + item.layout_box.dimensions.margin.top
        + padding.top
        + border.top;
}

// Note: Unit tests omitted because TextRenderer requires GPU context.
// Grid functionality is tested via integration tests.
