
#[derive(Debug, Default)]
pub struct History {
    entries: Vec<String>,
    current_index: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    pub fn push(&mut self, url: String) {
        if let Some(index) = self.current_index {
            self.entries.truncate(index + 1);
        }

        self.entries.push(url);
        self.current_index = Some(self.entries.len() - 1);
    }

    pub fn current(&self) -> Option<&str> {
        self.current_index.map(|i| self.entries[i].as_str())
    }

    pub fn can_go_back(&self) -> bool {
        self.current_index.map(|i| i > 0).unwrap_or(false)
    }

    pub fn can_go_forward(&self) -> bool {
        self.current_index
            .map(|i| i < self.entries.len() - 1)
            .unwrap_or(false)
    }

    pub fn go_back(&mut self) -> Option<&str> {
        if self.can_go_back() {
            self.current_index = self.current_index.map(|i| i - 1);
            self.current()
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<&str> {
        if self.can_go_forward() {
            self.current_index = self.current_index.map(|i| i + 1);
            self.current()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_navigation() {
        let mut history = History::new();

        history.push("https://example.com".to_string());
        history.push("https://example.com/page1".to_string());
        history.push("https://example.com/page2".to_string());

        assert_eq!(history.current(), Some("https://example.com/page2"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.go_back();
        assert_eq!(history.current(), Some("https://example.com/page1"));

        history.go_back();
        assert_eq!(history.current(), Some("https://example.com"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        history.go_forward();
        assert_eq!(history.current(), Some("https://example.com/page1"));
    }
}
