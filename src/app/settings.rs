#[derive(Debug, Clone)]
pub struct BrowserSettings {
    pub js_enabled: bool,
    pub css_enabled: bool,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            js_enabled: true,
            css_enabled: true,
        }
    }
}
