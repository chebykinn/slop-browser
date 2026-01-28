use crate::dom::{Document, NodeId};

/// Combinators between compound selectors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    /// Space: `div p` - descendant
    Descendant,
    /// `>`: `div > p` - direct child
    Child,
}

/// Structural pseudo-classes
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    FirstChild,
    LastChild,
    NthChild(i32, i32), // An+B formula: (a, b)
    OnlyChild,
}

/// Attribute selector types
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeSelector {
    /// [attr] - attribute exists
    Exists(String),
    /// [attr="val"] - exact match
    Equals(String, String),
    /// [attr*="val"] - contains substring
    Contains(String, String),
    /// [attr^="val"] - starts with
    StartsWith(String, String),
    /// [attr$="val"] - ends with
    EndsWith(String, String),
    /// [attr~="val"] - word in space-separated list
    WordMatch(String, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    Universal,
    Tag(String),
    Class(String),
    Id(String),
    Attribute(AttributeSelector),
    PseudoClass(PseudoClass),
}

/// A compound selector is a sequence of simple selectors (no combinators)
/// e.g., `div.container#main:first-child`
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector {
    pub simple_selectors: Vec<SimpleSelector>,
}

impl CompoundSelector {
    pub fn new() -> Self {
        Self {
            simple_selectors: Vec::new(),
        }
    }

    pub fn matches(&self, document: &Document, node_id: NodeId) -> bool {
        let node = match document.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        let element = match node.as_element() {
            Some(e) => e,
            None => return false,
        };

        for selector in &self.simple_selectors {
            let matches = match selector {
                SimpleSelector::Universal => true,
                SimpleSelector::Tag(tag) => element.tag_name.eq_ignore_ascii_case(tag),
                SimpleSelector::Class(class) => element.classes().iter().any(|c| c == class),
                SimpleSelector::Id(id) => element.id() == Some(id.as_str()),
                SimpleSelector::Attribute(attr_sel) => match attr_sel {
                    AttributeSelector::Exists(attr) => element.get_attribute(attr).is_some(),
                    AttributeSelector::Equals(attr, val) => {
                        element.get_attribute(attr) == Some(val.as_str())
                    }
                    AttributeSelector::Contains(attr, val) => {
                        element.get_attribute(attr).map_or(false, |v| v.contains(val.as_str()))
                    }
                    AttributeSelector::StartsWith(attr, val) => {
                        element.get_attribute(attr).map_or(false, |v| v.starts_with(val.as_str()))
                    }
                    AttributeSelector::EndsWith(attr, val) => {
                        element.get_attribute(attr).map_or(false, |v| v.ends_with(val.as_str()))
                    }
                    AttributeSelector::WordMatch(attr, val) => {
                        element.get_attribute(attr).map_or(false, |v| {
                            v.split_whitespace().any(|word| word == val.as_str())
                        })
                    }
                },
                SimpleSelector::PseudoClass(pseudo) => match pseudo {
                    PseudoClass::FirstChild => document.element_index(node_id) == Some(1),
                    PseudoClass::LastChild => document.is_last_element_child(node_id),
                    PseudoClass::OnlyChild => {
                        document.element_index(node_id) == Some(1) && document.is_last_element_child(node_id)
                    }
                    PseudoClass::NthChild(a, b) => {
                        if let Some(index) = document.element_index(node_id) {
                            nth_child_matches(*a, *b, index as i32)
                        } else {
                            false
                        }
                    }
                },
            };
            if !matches {
                return false;
            }
        }

        true
    }

    pub fn specificity(&self) -> Specificity {
        let mut ids = 0;
        let mut classes = 0;
        let mut tags = 0;

        for selector in &self.simple_selectors {
            match selector {
                SimpleSelector::Id(_) => ids += 1,
                SimpleSelector::Class(_) | SimpleSelector::Attribute(_) | SimpleSelector::PseudoClass(_) => {
                    classes += 1
                }
                SimpleSelector::Tag(_) => tags += 1,
                SimpleSelector::Universal => {}
            }
        }

        Specificity { ids, classes, tags }
    }
}

impl Default for CompoundSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if element index matches An+B formula
fn nth_child_matches(a: i32, b: i32, index: i32) -> bool {
    if a == 0 {
        return index == b;
    }
    let diff = index - b;
    if a > 0 {
        diff >= 0 && diff % a == 0
    } else {
        diff <= 0 && diff % a == 0
    }
}

/// A complex selector is a sequence of compound selectors with combinators
/// e.g., `div > p.intro span`
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    /// Parts stored right-to-left: (subject, combinator to parent, parent, ...)
    /// The first element is the subject (rightmost in CSS), no combinator
    /// Subsequent elements have the combinator that connects them to the previous
    pub parts: Vec<(CompoundSelector, Option<Combinator>)>,
}

impl ComplexSelector {
    pub fn new(parts: Vec<(CompoundSelector, Option<Combinator>)>) -> Self {
        Self { parts }
    }

    pub fn from_compound(compound: CompoundSelector) -> Self {
        Self {
            parts: vec![(compound, None)],
        }
    }

    pub fn matches(&self, document: &Document, node_id: NodeId) -> bool {
        if self.parts.is_empty() {
            return false;
        }

        // First part (subject) must match the node
        if !self.parts[0].0.matches(document, node_id) {
            return false;
        }

        // If only one part, we're done
        if self.parts.len() == 1 {
            return true;
        }

        // Check ancestor chain for remaining parts
        self.match_ancestors(document, node_id, 1)
    }

    fn match_ancestors(&self, document: &Document, node_id: NodeId, part_index: usize) -> bool {
        if part_index >= self.parts.len() {
            return true;
        }

        let (compound, combinator) = &self.parts[part_index];
        let combinator = combinator.unwrap_or(Combinator::Descendant);

        match combinator {
            Combinator::Child => {
                // Must match direct parent
                if let Some(parent_id) = document.parent(node_id) {
                    if compound.matches(document, parent_id) {
                        return self.match_ancestors(document, parent_id, part_index + 1);
                    }
                }
                false
            }
            Combinator::Descendant => {
                // Match any ancestor
                let mut current = document.parent(node_id);
                while let Some(ancestor_id) = current {
                    if compound.matches(document, ancestor_id) {
                        if self.match_ancestors(document, ancestor_id, part_index + 1) {
                            return true;
                        }
                    }
                    current = document.parent(ancestor_id);
                }
                false
            }
        }
    }

    pub fn specificity(&self) -> Specificity {
        let mut total = Specificity::new();
        for (compound, _) in &self.parts {
            let spec = compound.specificity();
            total.ids += spec.ids;
            total.classes += spec.classes;
            total.tags += spec.tags;
        }
        total
    }

    /// Get the rightmost (subject) compound selector for indexing
    pub fn subject(&self) -> Option<&CompoundSelector> {
        self.parts.first().map(|(c, _)| c)
    }
}

/// Legacy Selector type for backward compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub complex: ComplexSelector,
}

impl Selector {
    pub fn new() -> Self {
        Self {
            complex: ComplexSelector::new(vec![]),
        }
    }

    pub fn tag(tag_name: &str) -> Self {
        let mut compound = CompoundSelector::new();
        compound.simple_selectors.push(SimpleSelector::Tag(tag_name.to_lowercase()));
        Self {
            complex: ComplexSelector::from_compound(compound),
        }
    }

    pub fn class(class_name: &str) -> Self {
        let mut compound = CompoundSelector::new();
        compound.simple_selectors.push(SimpleSelector::Class(class_name.to_string()));
        Self {
            complex: ComplexSelector::from_compound(compound),
        }
    }

    pub fn id(id: &str) -> Self {
        let mut compound = CompoundSelector::new();
        compound.simple_selectors.push(SimpleSelector::Id(id.to_string()));
        Self {
            complex: ComplexSelector::from_compound(compound),
        }
    }

    pub fn matches(&self, document: &Document, node_id: NodeId) -> bool {
        self.complex.matches(document, node_id)
    }

    pub fn specificity(&self) -> Specificity {
        self.complex.specificity()
    }

    /// For backward compatibility
    pub fn simple_selectors(&self) -> Vec<&SimpleSelector> {
        self.complex
            .parts
            .iter()
            .flat_map(|(c, _)| c.simple_selectors.iter())
            .collect()
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub tags: u32,
}

impl Specificity {
    pub fn new() -> Self {
        Self {
            ids: 0,
            classes: 0,
            tags: 0,
        }
    }
}

impl Default for Specificity {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a CSS selector string
pub fn parse_selector(input: &str) -> Option<Selector> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut parts: Vec<(CompoundSelector, Option<Combinator>)> = Vec::new();
    let mut current_compound = CompoundSelector::new();
    let mut pending_combinator: Option<Combinator> = None;

    let mut chars = input.chars().peekable();
    let mut current = String::new();
    let mut selector_type = SelectorType::Tag;

    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                // Check if next non-space is '>'
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                if chars.peek() == Some(&'>') {
                    // Will be handled by '>' case
                    continue;
                }
                // It's a descendant combinator
                if !current_compound.simple_selectors.is_empty() {
                    parts.push((current_compound, pending_combinator));
                    current_compound = CompoundSelector::new();
                    pending_combinator = Some(Combinator::Descendant);
                }
            }
            '>' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                // Skip trailing spaces
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                if !current_compound.simple_selectors.is_empty() {
                    parts.push((current_compound, pending_combinator));
                    current_compound = CompoundSelector::new();
                    pending_combinator = Some(Combinator::Child);
                }
            }
            '.' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                selector_type = SelectorType::Class;
            }
            '#' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                selector_type = SelectorType::Id;
            }
            '[' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                // Parse attribute selector
                let mut attr_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    }
                    attr_str.push(chars.next().unwrap());
                }
                if let Some(attr_sel) = parse_attribute_selector(&attr_str) {
                    current_compound.simple_selectors.push(SimpleSelector::Attribute(attr_sel));
                }
            }
            ':' => {
                flush_current(&mut current, &mut selector_type, &mut current_compound);
                // Parse pseudo-class
                let mut pseudo_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '>' || c == '.' || c == '#' || c == '[' || c == ':' {
                        break;
                    }
                    pseudo_str.push(chars.next().unwrap());
                }
                if let Some(pseudo) = parse_pseudo_class(&pseudo_str) {
                    current_compound.simple_selectors.push(SimpleSelector::PseudoClass(pseudo));
                }
            }
            '*' if current.is_empty() => {
                current_compound.simple_selectors.push(SimpleSelector::Universal);
            }
            _ => {
                current.push(c);
            }
        }
    }

    flush_current(&mut current, &mut selector_type, &mut current_compound);

    if !current_compound.simple_selectors.is_empty() {
        parts.push((current_compound, pending_combinator));
    }

    if parts.is_empty() {
        return None;
    }

    // Reverse to get right-to-left order (subject first)
    parts.reverse();

    // After reversal, we need to shift combinators:
    // - parts[0] (subject) should have None (no combinator to its right)
    // - parts[i] for i > 0 should have the combinator from parts[i-1] before shift
    // This is because the combinator was stored with the part AFTER it in original order
    let combinators: Vec<Option<Combinator>> = parts.iter().map(|(_, c)| *c).collect();
    for i in 0..parts.len() {
        if i == 0 {
            parts[i].1 = None;
        } else {
            parts[i].1 = combinators[i - 1];
        }
    }

    Some(Selector {
        complex: ComplexSelector::new(parts),
    })
}

enum SelectorType {
    Tag,
    Class,
    Id,
}

fn flush_current(current: &mut String, selector_type: &mut SelectorType, compound: &mut CompoundSelector) {
    if !current.is_empty() {
        match selector_type {
            SelectorType::Tag => {
                compound.simple_selectors.push(SimpleSelector::Tag(current.to_lowercase()));
            }
            SelectorType::Class => {
                compound.simple_selectors.push(SimpleSelector::Class(current.clone()));
            }
            SelectorType::Id => {
                compound.simple_selectors.push(SimpleSelector::Id(current.clone()));
            }
        }
        current.clear();
    }
    *selector_type = SelectorType::Tag;
}

fn parse_attribute_selector(input: &str) -> Option<AttributeSelector> {
    let input = input.trim();

    // Check for operators
    if let Some(pos) = input.find("*=") {
        let attr = input[..pos].trim().to_string();
        let val = extract_attr_value(&input[pos + 2..]);
        return Some(AttributeSelector::Contains(attr, val));
    }
    if let Some(pos) = input.find("^=") {
        let attr = input[..pos].trim().to_string();
        let val = extract_attr_value(&input[pos + 2..]);
        return Some(AttributeSelector::StartsWith(attr, val));
    }
    if let Some(pos) = input.find("$=") {
        let attr = input[..pos].trim().to_string();
        let val = extract_attr_value(&input[pos + 2..]);
        return Some(AttributeSelector::EndsWith(attr, val));
    }
    if let Some(pos) = input.find("~=") {
        let attr = input[..pos].trim().to_string();
        let val = extract_attr_value(&input[pos + 2..]);
        return Some(AttributeSelector::WordMatch(attr, val));
    }
    if let Some(pos) = input.find('=') {
        let attr = input[..pos].trim().to_string();
        let val = extract_attr_value(&input[pos + 1..]);
        return Some(AttributeSelector::Equals(attr, val));
    }

    // Just attribute existence
    if !input.is_empty() {
        return Some(AttributeSelector::Exists(input.to_string()));
    }

    None
}

fn extract_attr_value(input: &str) -> String {
    let input = input.trim();
    // Remove quotes if present
    if (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''))
    {
        input[1..input.len() - 1].to_string()
    } else {
        input.to_string()
    }
}

fn parse_pseudo_class(input: &str) -> Option<PseudoClass> {
    if input == "first-child" {
        return Some(PseudoClass::FirstChild);
    }
    if input == "last-child" {
        return Some(PseudoClass::LastChild);
    }
    if input == "only-child" {
        return Some(PseudoClass::OnlyChild);
    }
    if input.starts_with("nth-child(") && input.ends_with(')') {
        let formula = &input[10..input.len() - 1];
        return parse_nth_formula(formula).map(|(a, b)| PseudoClass::NthChild(a, b));
    }

    None
}

fn parse_nth_formula(input: &str) -> Option<(i32, i32)> {
    let input = input.trim().to_lowercase();

    // Handle keywords
    if input == "odd" {
        return Some((2, 1));
    }
    if input == "even" {
        return Some((2, 0));
    }

    // Handle simple number
    if let Ok(n) = input.parse::<i32>() {
        return Some((0, n));
    }

    // Handle An+B formula
    let input = input.replace(" ", "");

    if let Some(n_pos) = input.find('n') {
        let a_part = &input[..n_pos];
        let a = if a_part.is_empty() || a_part == "+" {
            1
        } else if a_part == "-" {
            -1
        } else {
            a_part.parse::<i32>().unwrap_or(1)
        };

        let b_part = &input[n_pos + 1..];
        let b = if b_part.is_empty() {
            0
        } else {
            b_part.parse::<i32>().unwrap_or(0)
        };

        return Some((a, b));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_selector() {
        let sel = parse_selector("div").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Tag("div".to_string())]
        );
    }

    #[test]
    fn test_parse_class_selector() {
        let sel = parse_selector(".container").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Class("container".to_string())]
        );
    }

    #[test]
    fn test_parse_id_selector() {
        let sel = parse_selector("#main").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Id("main".to_string())]
        );
    }

    #[test]
    fn test_parse_compound_selector() {
        let sel = parse_selector("div.container#main").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![
                SimpleSelector::Tag("div".to_string()),
                SimpleSelector::Class("container".to_string()),
                SimpleSelector::Id("main".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_descendant_combinator() {
        let sel = parse_selector("div p").unwrap();
        assert_eq!(sel.complex.parts.len(), 2);
        // First is subject (p)
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Tag("p".to_string())]
        );
        assert_eq!(sel.complex.parts[0].1, None);
        // Second is ancestor (div) with descendant combinator
        assert_eq!(
            sel.complex.parts[1].0.simple_selectors,
            vec![SimpleSelector::Tag("div".to_string())]
        );
        assert_eq!(sel.complex.parts[1].1, Some(Combinator::Descendant));
    }

    #[test]
    fn test_parse_child_combinator() {
        let sel = parse_selector("div > p").unwrap();
        assert_eq!(sel.complex.parts.len(), 2);
        // First is subject (p)
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Tag("p".to_string())]
        );
        // Second is parent (div) with child combinator
        assert_eq!(
            sel.complex.parts[1].0.simple_selectors,
            vec![SimpleSelector::Tag("div".to_string())]
        );
        assert_eq!(sel.complex.parts[1].1, Some(Combinator::Child));
    }

    #[test]
    fn test_parse_pseudo_class_first_child() {
        let sel = parse_selector("p:first-child").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![
                SimpleSelector::Tag("p".to_string()),
                SimpleSelector::PseudoClass(PseudoClass::FirstChild),
            ]
        );
    }

    #[test]
    fn test_parse_pseudo_class_nth_child() {
        let sel = parse_selector("li:nth-child(2n+1)").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![
                SimpleSelector::Tag("li".to_string()),
                SimpleSelector::PseudoClass(PseudoClass::NthChild(2, 1)),
            ]
        );
    }

    #[test]
    fn test_parse_attribute_exists() {
        let sel = parse_selector("[disabled]").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Attribute(AttributeSelector::Exists("disabled".to_string()))]
        );
    }

    #[test]
    fn test_parse_attribute_equals() {
        let sel = parse_selector("[type=\"text\"]").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Attribute(AttributeSelector::Equals(
                "type".to_string(),
                "text".to_string()
            ))]
        );
    }

    #[test]
    fn test_parse_attribute_contains() {
        let sel = parse_selector("[class*=\"btn\"]").unwrap();
        assert_eq!(sel.complex.parts.len(), 1);
        assert_eq!(
            sel.complex.parts[0].0.simple_selectors,
            vec![SimpleSelector::Attribute(AttributeSelector::Contains(
                "class".to_string(),
                "btn".to_string()
            ))]
        );
    }

    #[test]
    fn test_nth_formula_odd() {
        assert_eq!(parse_nth_formula("odd"), Some((2, 1)));
    }

    #[test]
    fn test_nth_formula_even() {
        assert_eq!(parse_nth_formula("even"), Some((2, 0)));
    }

    #[test]
    fn test_nth_formula_number() {
        assert_eq!(parse_nth_formula("3"), Some((0, 3)));
    }

    #[test]
    fn test_nth_formula_an_plus_b() {
        assert_eq!(parse_nth_formula("2n+1"), Some((2, 1)));
        assert_eq!(parse_nth_formula("3n-2"), Some((3, -2)));
        assert_eq!(parse_nth_formula("n"), Some((1, 0)));
        assert_eq!(parse_nth_formula("-n+3"), Some((-1, 3)));
    }

    #[test]
    fn test_specificity() {
        let sel = parse_selector("div.container#main").unwrap();
        let spec = sel.specificity();
        assert_eq!(spec.ids, 1);
        assert_eq!(spec.classes, 1);
        assert_eq!(spec.tags, 1);
    }

    #[test]
    fn test_complex_selector_specificity() {
        let sel = parse_selector("div > p.intro").unwrap();
        let spec = sel.specificity();
        assert_eq!(spec.ids, 0);
        assert_eq!(spec.classes, 1);
        assert_eq!(spec.tags, 2);
    }
}
