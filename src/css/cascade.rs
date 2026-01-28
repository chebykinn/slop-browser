use super::computed::ComputedStyle;
use super::index::{IndexedRule, SelectorIndex};
use super::selector::Specificity;
use super::stylesheet::Stylesheet;
use crate::dom::{Document, NodeId};
use std::collections::HashMap;
use std::rc::Rc;

pub struct StyleComputer {
    stylesheets: Vec<Rc<Stylesheet>>,
    /// Lazily-built selector index for O(1) rule lookups
    selector_index: Option<SelectorIndex>,
    computed_styles: HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
    viewport_height: f32,
}

impl StyleComputer {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            stylesheets: Vec::new(),
            selector_index: None,
            computed_styles: HashMap::new(),
            viewport_width,
            viewport_height,
        }
    }

    pub fn add_stylesheet(&mut self, stylesheet: Rc<Stylesheet>) {
        self.stylesheets.push(stylesheet);
        // Invalidate the index when stylesheets change
        self.selector_index = None;
    }

    pub fn clear_stylesheets(&mut self) {
        self.stylesheets.clear();
        self.selector_index = None;
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Ensure the selector index is built
    fn ensure_index(&mut self) {
        if self.selector_index.is_none() {
            self.selector_index = Some(SelectorIndex::build(&self.stylesheets));
        }
    }

    pub fn compute_styles(&mut self, document: &Document) {
        self.computed_styles.clear();
        self.ensure_index();
        self.compute_node_styles(document, document.root, None);
    }

    fn compute_node_styles(
        &mut self,
        document: &Document,
        node_id: NodeId,
        parent_style: Option<&ComputedStyle>,
    ) {
        let node = match document.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        let style = if let Some(element) = node.as_element() {
            let mut style = ComputedStyle::for_tag(&element.tag_name);

            if let Some(parent) = parent_style {
                style.color = parent.color;
                style.font_size = parent.font_size;
                style.line_height = parent.line_height;
            }

            // Use the selector index for O(1) candidate lookup
            let mut matching_rules: Vec<(Specificity, usize, &IndexedRule)> = Vec::new();

            if let Some(index) = &self.selector_index {
                let candidates = index.get_candidate_rules(
                    element.id(),
                    element.classes(),
                    &element.tag_name,
                );

                for indexed_rule in candidates {
                    // Verify the selector actually matches (handles compound selectors)
                    if indexed_rule.selector.matches(document, node_id) {
                        matching_rules.push((
                            indexed_rule.specificity,
                            indexed_rule.source_order,
                            indexed_rule,
                        ));
                    }
                }
            }

            // Sort by specificity, then by source order for stable ordering
            matching_rules.sort_by_key(|(spec, order, _)| (*spec, *order));

            let parent_font_size = parent_style.map(|p| p.font_size).unwrap_or(16.0);

            // Apply presentational HTML attributes (lowest priority, before CSS)
            self.apply_presentational_attributes(&mut style, element, parent_font_size);

            // First apply all non-important declarations
            for (_, _, indexed_rule) in &matching_rules {
                for decl in &indexed_rule.rule.declarations {
                    if !decl.important {
                        style.apply_value(
                            &decl.property,
                            &decl.value,
                            parent_font_size,
                            self.viewport_width,
                            self.viewport_height,
                        );
                    }
                }
            }

            // Then apply all !important declarations (they override normal ones)
            for (_, _, indexed_rule) in &matching_rules {
                for decl in &indexed_rule.rule.declarations {
                    if decl.important {
                        style.apply_value(
                            &decl.property,
                            &decl.value,
                            parent_font_size,
                            self.viewport_width,
                            self.viewport_height,
                        );
                    }
                }
            }

            if let Some(style_attr) = element.get_attribute("style") {
                let inline_styles = super::parser::parse_css(&format!("* {{ {} }}", style_attr));
                for rule in &inline_styles.rules {
                    for decl in &rule.declarations {
                        style.apply_value(
                            &decl.property,
                            &decl.value,
                            parent_font_size,
                            self.viewport_width,
                            self.viewport_height,
                        );
                    }
                }
            }

            style
        } else {
            parent_style.cloned().unwrap_or_default()
        };

        self.computed_styles.insert(node_id, style.clone());

        let children: Vec<_> = document.children(node_id).to_vec();
        for child_id in children {
            self.compute_node_styles(document, child_id, Some(&style));
        }
    }

    pub fn get_style(&self, node_id: NodeId) -> Option<&ComputedStyle> {
        self.computed_styles.get(&node_id)
    }

    pub fn get_style_mut(&mut self, node_id: NodeId) -> Option<&mut ComputedStyle> {
        self.computed_styles.get_mut(&node_id)
    }

    /// Apply HTML presentational attributes to the computed style.
    /// These have the lowest specificity and can be overridden by CSS.
    fn apply_presentational_attributes(
        &self,
        style: &mut ComputedStyle,
        element: &crate::dom::node::ElementData,
        parent_font_size: f32,
    ) {
        use crate::render::painter::Color;

        // bgcolor attribute (used by HN for table backgrounds)
        if let Some(bgcolor) = element.attributes.get("bgcolor") {
            if let Some(color) = Color::from_hex(bgcolor) {
                style.background_color = color;
            } else {
                // Try as named color
                let value = super::stylesheet::Value::Keyword(bgcolor.to_string());
                if let Some(color) = value.to_color() {
                    style.background_color = color;
                }
            }
        }

        // width attribute
        if let Some(width) = element.attributes.get("width") {
            if let Ok(px) = width.trim_end_matches("px").parse::<f32>() {
                style.width = Some(super::computed::LengthOrPercentage::Px(px));
            } else if let Ok(percent) = width.trim_end_matches('%').parse::<f32>() {
                style.width = Some(super::computed::LengthOrPercentage::Percent(percent));
            }
        }

        // height attribute
        if let Some(height) = element.attributes.get("height") {
            if let Ok(px) = height.trim_end_matches("px").parse::<f32>() {
                style.height = Some(super::computed::LengthOrPercentage::Px(px));
            }
        }

        // border attribute
        if let Some(border) = element.attributes.get("border") {
            if let Ok(px) = border.parse::<f32>() {
                style.border_top_width = px;
                style.border_right_width = px;
                style.border_bottom_width = px;
                style.border_left_width = px;
                // Also set border color to black if not set
                if style.border_color == Color::TRANSPARENT {
                    style.border_color = Color::BLACK;
                }
            }
        }

        // cellpadding attribute (for tables)
        if let Some(cellpadding) = element.attributes.get("cellpadding") {
            if let Ok(px) = cellpadding.parse::<f32>() {
                style.padding_top = px;
                style.padding_right = px;
                style.padding_bottom = px;
                style.padding_left = px;
            }
        }

        // cellspacing attribute (for tables)
        if let Some(cellspacing) = element.attributes.get("cellspacing") {
            if let Ok(px) = cellspacing.parse::<f32>() {
                style.border_spacing = px;
            }
        }

        // valign attribute (vertical alignment in table cells)
        if let Some(valign) = element.attributes.get("valign") {
            style.vertical_align = match valign.to_lowercase().as_str() {
                "top" => super::computed::VerticalAlign::Top,
                "middle" => super::computed::VerticalAlign::Middle,
                "bottom" => super::computed::VerticalAlign::Bottom,
                "baseline" => super::computed::VerticalAlign::Baseline,
                _ => style.vertical_align,
            };
        }

        // align attribute (text alignment)
        if let Some(align) = element.attributes.get("align") {
            style.text_align = match align.to_lowercase().as_str() {
                "left" => super::computed::TextAlign::Left,
                "center" => super::computed::TextAlign::Center,
                "right" => super::computed::TextAlign::Right,
                "justify" => super::computed::TextAlign::Justify,
                _ => style.text_align,
            };
        }

        // color attribute (for font elements)
        if let Some(color_attr) = element.attributes.get("color") {
            if let Some(color) = Color::from_hex(color_attr) {
                style.color = color;
            } else {
                let value = super::stylesheet::Value::Keyword(color_attr.to_string());
                if let Some(color) = value.to_color() {
                    style.color = color;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parse_css;
    use crate::dom::parse_html;

    #[test]
    fn test_cascade_basic() {
        let html = r#"<html><body><p class="red">Hello</p></body></html>"#;
        let doc = parse_html(html);

        let css = ".red { color: red; }";
        let stylesheet = Rc::new(parse_css(css));

        let mut computer = StyleComputer::new(800.0, 600.0);
        computer.add_stylesheet(stylesheet);
        computer.compute_styles(&doc);

        let p_elements = doc.get_elements_by_tag_name("p");
        let style = computer.get_style(p_elements[0]).unwrap();

        assert_eq!(style.color.r, 1.0);
        assert_eq!(style.color.g, 0.0);
        assert_eq!(style.color.b, 0.0);
    }

    #[test]
    fn test_specificity_order() {
        let html = r#"<html><body><p id="main" class="text">Hello</p></body></html>"#;
        let doc = parse_html(html);

        let css = r#"
            p { color: blue; }
            .text { color: green; }
            #main { color: red; }
        "#;
        let stylesheet = Rc::new(parse_css(css));

        let mut computer = StyleComputer::new(800.0, 600.0);
        computer.add_stylesheet(stylesheet);
        computer.compute_styles(&doc);

        let p_elements = doc.get_elements_by_tag_name("p");
        let style = computer.get_style(p_elements[0]).unwrap();

        assert_eq!(style.color.r, 1.0);
        assert_eq!(style.color.g, 0.0);
        assert_eq!(style.color.b, 0.0);
    }
}
