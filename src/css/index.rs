//! Selector indexing for efficient CSS rule matching.
//!
//! Instead of iterating through all rules for every element (O(n³)),
//! this module indexes rules by their most restrictive selector component
//! (id > class > tag > universal) for O(1) candidate lookup.

use super::selector::{Selector, SimpleSelector, Specificity};
use super::stylesheet::{Rule, Stylesheet};
use std::collections::HashMap;
use std::rc::Rc;

/// An indexed rule that references the original rule with its specificity.
#[derive(Clone)]
pub struct IndexedRule {
    /// Reference to the original rule
    pub rule: Rc<Rule>,
    /// The selector that matched (for specificity calculation)
    pub selector: Selector,
    /// Pre-computed specificity
    pub specificity: Specificity,
    /// Order in which the rule appeared (for stable sorting)
    pub source_order: usize,
}

/// Index structure for fast selector matching.
///
/// Rules are indexed by their most restrictive selector component:
/// - ID selectors go to by_id
/// - Class selectors go to by_class
/// - Tag selectors go to by_tag
/// - Universal selectors go to universal
///
/// When matching an element, we only need to check rules from:
/// - by_id[element.id]
/// - by_class[class] for each class in element.classes
/// - by_tag[element.tag_name]
/// - universal
pub struct SelectorIndex {
    /// Rules indexed by ID selector
    by_id: HashMap<String, Vec<IndexedRule>>,
    /// Rules indexed by class selector
    by_class: HashMap<String, Vec<IndexedRule>>,
    /// Rules indexed by tag selector
    by_tag: HashMap<String, Vec<IndexedRule>>,
    /// Rules with only universal selectors
    universal: Vec<IndexedRule>,
}

impl SelectorIndex {
    /// Build an index from a list of stylesheets.
    pub fn build(stylesheets: &[Rc<Stylesheet>]) -> Self {
        let mut by_id: HashMap<String, Vec<IndexedRule>> = HashMap::new();
        let mut by_class: HashMap<String, Vec<IndexedRule>> = HashMap::new();
        let mut by_tag: HashMap<String, Vec<IndexedRule>> = HashMap::new();
        let mut universal: Vec<IndexedRule> = Vec::new();

        let mut source_order = 0;

        for stylesheet in stylesheets {
            for rule in &stylesheet.rules {
                let rule_rc = Rc::new(rule.clone());

                for selector in &rule.selectors {
                    let indexed_rule = IndexedRule {
                        rule: Rc::clone(&rule_rc),
                        selector: selector.clone(),
                        specificity: selector.specificity(),
                        source_order,
                    };
                    source_order += 1;

                    // Index by the most restrictive selector component
                    let key = Self::get_index_key(selector);

                    match key {
                        IndexKey::Id(id) => {
                            by_id.entry(id).or_default().push(indexed_rule);
                        }
                        IndexKey::Class(class) => {
                            by_class.entry(class).or_default().push(indexed_rule);
                        }
                        IndexKey::Tag(tag) => {
                            by_tag.entry(tag).or_default().push(indexed_rule);
                        }
                        IndexKey::Universal => {
                            universal.push(indexed_rule);
                        }
                    }
                }
            }
        }

        Self {
            by_id,
            by_class,
            by_tag,
            universal,
        }
    }

    /// Determine the best index key for a selector.
    /// Priority: ID > Class > Tag > Universal
    fn get_index_key(selector: &Selector) -> IndexKey {
        // First, look for ID selectors (most specific)
        for simple in &selector.simple_selectors() {
            if let SimpleSelector::Id(id) = simple {
                return IndexKey::Id(id.clone());
            }
        }

        // Then, look for class selectors
        for simple in &selector.simple_selectors() {
            if let SimpleSelector::Class(class) = simple {
                return IndexKey::Class(class.clone());
            }
        }

        // Then, look for tag selectors
        for simple in &selector.simple_selectors() {
            if let SimpleSelector::Tag(tag) = simple {
                return IndexKey::Tag(tag.clone());
            }
        }

        // Fall back to universal
        IndexKey::Universal
    }

    /// Get candidate rules that might match an element.
    /// This returns rules from relevant buckets that need to be checked
    /// against the element for an actual match.
    pub fn get_candidate_rules<'a>(
        &'a self,
        id: Option<&str>,
        classes: &[String],
        tag_name: &str,
    ) -> Vec<&'a IndexedRule> {
        let mut candidates = Vec::new();

        // Add rules matching by ID
        if let Some(id) = id {
            if let Some(rules) = self.by_id.get(id) {
                candidates.extend(rules.iter());
            }
        }

        // Add rules matching by class
        for class in classes {
            if let Some(rules) = self.by_class.get(class) {
                candidates.extend(rules.iter());
            }
        }

        // Add rules matching by tag
        if let Some(rules) = self.by_tag.get(&tag_name.to_lowercase()) {
            candidates.extend(rules.iter());
        }

        // Add universal rules
        candidates.extend(self.universal.iter());

        candidates
    }

    /// Check if the index is empty (no rules indexed).
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
            && self.by_class.is_empty()
            && self.by_tag.is_empty()
            && self.universal.is_empty()
    }
}

/// Internal key type for indexing.
enum IndexKey {
    Id(String),
    Class(String),
    Tag(String),
    Universal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parse_css;

    #[test]
    fn test_index_by_id() {
        let css = "#main { color: red; }";
        let stylesheet = Rc::new(parse_css(css));
        let index = SelectorIndex::build(&[stylesheet]);

        assert!(index.by_id.contains_key("main"));
        assert_eq!(index.by_id.get("main").unwrap().len(), 1);
        assert!(index.by_class.is_empty());
        assert!(index.by_tag.is_empty());
        assert!(index.universal.is_empty());
    }

    #[test]
    fn test_index_by_class() {
        let css = ".container { margin: 10px; }";
        let stylesheet = Rc::new(parse_css(css));
        let index = SelectorIndex::build(&[stylesheet]);

        assert!(index.by_id.is_empty());
        assert!(index.by_class.contains_key("container"));
        assert_eq!(index.by_class.get("container").unwrap().len(), 1);
    }

    #[test]
    fn test_index_by_tag() {
        let css = "div { display: block; }";
        let stylesheet = Rc::new(parse_css(css));
        let index = SelectorIndex::build(&[stylesheet]);

        assert!(index.by_id.is_empty());
        assert!(index.by_class.is_empty());
        assert!(index.by_tag.contains_key("div"));
    }

    #[test]
    fn test_compound_selector_indexed_by_id() {
        // Compound selector div#main.container should be indexed by ID
        let css = "div#main.container { color: blue; }";
        let stylesheet = Rc::new(parse_css(css));
        let index = SelectorIndex::build(&[stylesheet]);

        // Should be indexed by ID (most specific)
        assert!(index.by_id.contains_key("main"));
        assert!(index.by_class.is_empty());
        assert!(index.by_tag.is_empty());
    }

    #[test]
    fn test_get_candidate_rules() {
        let css = r#"
            #main { color: red; }
            .container { margin: 10px; }
            div { display: block; }
            * { box-sizing: border-box; }
        "#;
        let stylesheet = Rc::new(parse_css(css));
        let index = SelectorIndex::build(&[stylesheet]);

        // Element with id="main", class="container", tag="div"
        let classes = vec!["container".to_string()];
        let candidates = index.get_candidate_rules(Some("main"), &classes, "div");

        // Should get 4 candidates: #main, .container, div, *
        assert_eq!(candidates.len(), 4);
    }

    #[test]
    fn test_multiple_stylesheets() {
        let css1 = ".a { color: red; }";
        let css2 = ".b { color: blue; }";
        let stylesheets = vec![
            Rc::new(parse_css(css1)),
            Rc::new(parse_css(css2)),
        ];
        let index = SelectorIndex::build(&stylesheets);

        assert!(index.by_class.contains_key("a"));
        assert!(index.by_class.contains_key("b"));
    }
}
