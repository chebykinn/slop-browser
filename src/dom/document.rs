use super::node::{Node, NodeData, NodeId};

#[derive(Debug)]
pub struct Document {
    nodes: Vec<Node>,
    pub root: NodeId,
}

impl Document {
    pub fn new() -> Self {
        let root = Node::new_document(0);
        Self {
            nodes: vec![root],
            root: 0,
        }
    }

    pub fn create_element(&mut self, tag_name: &str) -> NodeId {
        let id = self.nodes.len();
        let node = Node::new_element(id, tag_name.to_lowercase());
        self.nodes.push(node);
        id
    }

    pub fn create_text(&mut self, content: &str) -> NodeId {
        let id = self.nodes.len();
        let node = Node::new_text(id, content.to_string());
        self.nodes.push(node);
        id
    }

    pub fn create_comment(&mut self, content: &str) -> NodeId {
        let id = self.nodes.len();
        let node = Node::new_comment(id, content.to_string());
        self.nodes.push(node);
        id
    }

    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        if parent_id < self.nodes.len() && child_id < self.nodes.len() {
            self.nodes[child_id].parent = Some(parent_id);
            self.nodes[parent_id].children.push(child_id);
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn set_attribute(&mut self, node_id: NodeId, name: &str, value: &str) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            if let Some(elem) = node.as_element_mut() {
                elem.attributes.insert(name.to_string(), value.to_string());
            }
        }
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        for node in &self.nodes {
            if let Some(elem) = node.as_element() {
                if elem.id() == Some(id) {
                    return Some(node.id);
                }
            }
        }
        None
    }

    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        let tag_lower = tag_name.to_lowercase();
        self.nodes
            .iter()
            .filter_map(|node| {
                if node.tag_name() == Some(&tag_lower) {
                    Some(node.id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|node| {
                if let Some(elem) = node.as_element() {
                    if elem.classes().iter().any(|c| c == class_name) {
                        return Some(node.id);
                    }
                }
                None
            })
            .collect()
    }

    pub fn get_body(&self) -> Option<NodeId> {
        self.get_elements_by_tag_name("body").first().copied()
    }

    pub fn get_head(&self) -> Option<NodeId> {
        self.get_elements_by_tag_name("head").first().copied()
    }

    pub fn children(&self, node_id: NodeId) -> &[NodeId] {
        self.nodes
            .get(node_id)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    pub fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes.get(node_id).and_then(|n| n.parent)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub fn get_text_content(&self, node_id: NodeId) -> String {
        let mut result = String::new();
        self.collect_text_content(node_id, &mut result);
        result
    }

    fn collect_text_content(&self, node_id: NodeId, result: &mut String) {
        if let Some(node) = self.get_node(node_id) {
            match &node.data {
                NodeData::Text(text) => result.push_str(text),
                _ => {
                    for &child_id in &node.children {
                        self.collect_text_content(child_id, result);
                    }
                }
            }
        }
    }

    pub fn set_text_content(&mut self, node_id: NodeId, content: &str) {
        if let Some(node) = self.get_node_mut(node_id) {
            node.children.clear();
        }
        let text_id = self.create_text(content);
        self.append_child(node_id, text_id);
    }

    /// Get the 1-based index of an element among its element siblings
    /// Returns None if node is not an element or has no parent
    pub fn element_index(&self, node_id: NodeId) -> Option<usize> {
        let node = self.get_node(node_id)?;

        // Must be an element
        if !node.is_element() {
            return None;
        }

        let parent_id = node.parent?;
        let siblings = self.children(parent_id);

        let mut element_index = 0;
        for &sibling_id in siblings {
            if let Some(sibling) = self.get_node(sibling_id) {
                if sibling.is_element() {
                    element_index += 1;
                    if sibling_id == node_id {
                        return Some(element_index);
                    }
                }
            }
        }

        None
    }

    /// Check if node is the last element child of its parent
    pub fn is_last_element_child(&self, node_id: NodeId) -> bool {
        let node = match self.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        if !node.is_element() {
            return false;
        }

        let parent_id = match node.parent {
            Some(p) => p,
            None => return false,
        };

        let siblings = self.children(parent_id);

        // Find the last element sibling
        for &sibling_id in siblings.iter().rev() {
            if let Some(sibling) = self.get_node(sibling_id) {
                if sibling.is_element() {
                    return sibling_id == node_id;
                }
            }
        }

        false
    }

    /// Get element children only (filtering out text/comment nodes)
    pub fn element_children(&self, node_id: NodeId) -> Vec<NodeId> {
        self.children(node_id)
            .iter()
            .filter(|&&child_id| {
                self.get_node(child_id)
                    .map(|n| n.is_element())
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }

    /// Get all ancestors of a node (parent, grandparent, etc.)
    pub fn ancestors(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = self.parent(node_id);
        while let Some(ancestor_id) = current {
            result.push(ancestor_id);
            current = self.parent(ancestor_id);
        }
        result
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
