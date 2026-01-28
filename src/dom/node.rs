use std::cell::OnceCell;
use std::collections::HashMap;

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub data: NodeData,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Document,
    Element(ElementData),
    Text(String),
    Comment(String),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    /// Cached parsed classes for efficient selector matching
    cached_classes: OnceCell<Vec<String>>,
}

impl Clone for ElementData {
    fn clone(&self) -> Self {
        Self {
            tag_name: self.tag_name.clone(),
            attributes: self.attributes.clone(),
            cached_classes: OnceCell::new(), // Reset cache on clone
        }
    }
}

impl ElementData {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_name,
            attributes: HashMap::new(),
            cached_classes: OnceCell::new(),
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.attributes.get("id").map(|s| s.as_str())
    }

    /// Returns cached parsed classes. Uses OnceCell for lazy initialization.
    pub fn classes(&self) -> &[String] {
        self.cached_classes.get_or_init(|| {
            self.attributes
                .get("class")
                .map(|s| s.split_whitespace().map(|c| c.to_string()).collect())
                .unwrap_or_default()
        })
    }

    /// Invalidates the class cache. Call this when the class attribute changes.
    pub fn invalidate_class_cache(&mut self) {
        self.cached_classes.take();
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }
}

impl Node {
    pub fn new_document(id: NodeId) -> Self {
        Self {
            id,
            data: NodeData::Document,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_element(id: NodeId, tag_name: String) -> Self {
        Self {
            id,
            data: NodeData::Element(ElementData::new(tag_name)),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_text(id: NodeId, content: String) -> Self {
        Self {
            id,
            data: NodeData::Text(content),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_comment(id: NodeId, content: String) -> Self {
        Self {
            id,
            data: NodeData::Comment(content),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self.data, NodeData::Text(_))
    }

    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.data {
            NodeData::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn tag_name(&self) -> Option<&str> {
        self.as_element().map(|e| e.tag_name.as_str())
    }
}
