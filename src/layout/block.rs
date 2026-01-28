use super::box_model::BoxDimensions;
use super::tree::LayoutBox;

pub fn calculate_block_width(
    layout_box: &mut LayoutBox,
    containing_width: f32,
) {
    let style = &layout_box.style;

    let margin_left = style.margin_left;
    let margin_right = style.margin_right;
    let padding_left = style.padding_left;
    let padding_right = style.padding_right;
    let border_left = style.border_left_width;
    let border_right = style.border_right_width;

    let total_horizontal = margin_left + margin_right + padding_left + padding_right + border_left + border_right;

    let content_width = if let Some(w) = style.width {
        w
    } else {
        (containing_width - total_horizontal).max(0.0)
    };

    layout_box.dimensions.content.width = content_width;
    layout_box.dimensions.margin.left = margin_left;
    layout_box.dimensions.margin.right = margin_right;
    layout_box.dimensions.padding.left = padding_left;
    layout_box.dimensions.padding.right = padding_right;
    layout_box.dimensions.border.left = border_left;
    layout_box.dimensions.border.right = border_right;
}

pub fn calculate_block_position(
    layout_box: &mut LayoutBox,
    containing_block: &BoxDimensions,
    cursor_y: f32,
) {
    let style = &layout_box.style;

    layout_box.dimensions.margin.top = style.margin_top;
    layout_box.dimensions.margin.bottom = style.margin_bottom;
    layout_box.dimensions.padding.top = style.padding_top;
    layout_box.dimensions.padding.bottom = style.padding_bottom;
    layout_box.dimensions.border.top = style.border_top_width;
    layout_box.dimensions.border.bottom = style.border_bottom_width;

    layout_box.dimensions.content.x = containing_block.content.x
        + layout_box.dimensions.margin.left
        + layout_box.dimensions.border.left
        + layout_box.dimensions.padding.left;

    layout_box.dimensions.content.y = cursor_y
        + layout_box.dimensions.margin.top
        + layout_box.dimensions.border.top
        + layout_box.dimensions.padding.top;
}

pub fn calculate_block_height(layout_box: &mut LayoutBox) {
    if let Some(h) = layout_box.style.height {
        layout_box.dimensions.content.height = h;
    }
}
