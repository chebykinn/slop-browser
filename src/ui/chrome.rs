use crate::app::BrowserSettings;
use crate::render::painter::{Color, DisplayList, Rect};
use sdl2::keyboard::Keycode;

pub enum ChromeAction {
    Back,
    Forward,
    Navigate(String),
    FocusUrlBar,
    ToggleJs,
    ToggleCss,
    Stop,
    Reload,
}

pub struct Chrome {
    pub width: f32,
    pub height: f32,
    pub url_text: String,
    pub url_bar_focused: bool,
    pub back_button: Rect,
    pub forward_button: Rect,
    pub stop_button: Rect,
    pub url_bar: Rect,
    pub js_toggle: Rect,
    pub css_toggle: Rect,
    pub js_enabled: bool,
    pub css_enabled: bool,
    pub loading: bool,
    pub progress: Option<f32>, // 0.0 to 1.0
    /// Frames per second
    pub fps: f32,
    /// Last frame render time in milliseconds
    pub render_time_ms: f32,
}

impl Chrome {
    pub fn new(width: f32, height: f32, settings: &BrowserSettings) -> Self {
        let button_size = 30.0;
        let button_y = (height - button_size) / 2.0;
        let margin = 10.0;
        let toggle_width = 30.0;
        let toggle_spacing = 5.0;

        // Calculate positions for toggle buttons on the right
        let css_toggle_x = width - margin - toggle_width;
        let js_toggle_x = css_toggle_x - toggle_width - toggle_spacing;

        // URL bar ends before the toggle buttons
        let url_bar_end = js_toggle_x - margin;

        Self {
            width,
            height,
            url_text: String::new(),
            url_bar_focused: true,
            back_button: Rect::new(margin, button_y, button_size, button_size),
            forward_button: Rect::new(margin + button_size + 5.0, button_y, button_size, button_size),
            stop_button: Rect::new(margin + button_size * 2.0 + 10.0, button_y, button_size, button_size),
            url_bar: Rect::new(
                margin + button_size * 3.0 + 20.0,
                button_y,
                url_bar_end - (margin + button_size * 3.0 + 20.0),
                button_size,
            ),
            js_toggle: Rect::new(js_toggle_x, button_y, toggle_width, button_size),
            css_toggle: Rect::new(css_toggle_x, button_y, toggle_width, button_size),
            js_enabled: settings.js_enabled,
            css_enabled: settings.css_enabled,
            loading: false,
            progress: None,
            fps: 0.0,
            render_time_ms: 0.0,
        }
    }

    pub fn set_render_stats(&mut self, fps: f32, render_time_ms: f32) {
        self.fps = fps;
        self.render_time_ms = render_time_ms;
    }

    pub fn update_toggle_state(&mut self, settings: &BrowserSettings) {
        self.js_enabled = settings.js_enabled;
        self.css_enabled = settings.css_enabled;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_progress(&mut self, progress: Option<f32>) {
        self.progress = progress;
    }

    pub fn resize(&mut self, width: f32) {
        self.width = width;
        let margin = 10.0;
        let toggle_width = 30.0;
        let toggle_spacing = 5.0;

        // Reposition toggle buttons
        let css_toggle_x = width - margin - toggle_width;
        let js_toggle_x = css_toggle_x - toggle_width - toggle_spacing;
        self.js_toggle.x = js_toggle_x;
        self.css_toggle.x = css_toggle_x;

        // URL bar ends before the toggle buttons
        let url_bar_end = js_toggle_x - margin;
        self.url_bar.width = url_bar_end - self.url_bar.x;
    }

    pub fn set_url(&mut self, url: &str) {
        self.url_text = url.to_string();
    }

    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<ChromeAction> {
        if self.back_button.contains(x, y) {
            return Some(ChromeAction::Back);
        }

        if self.forward_button.contains(x, y) {
            return Some(ChromeAction::Forward);
        }

        if self.stop_button.contains(x, y) {
            if self.loading {
                return Some(ChromeAction::Stop);
            } else {
                return Some(ChromeAction::Reload);
            }
        }

        if self.js_toggle.contains(x, y) {
            return Some(ChromeAction::ToggleJs);
        }

        if self.css_toggle.contains(x, y) {
            return Some(ChromeAction::ToggleCss);
        }

        if self.url_bar.contains(x, y) {
            self.url_bar_focused = true;
            return Some(ChromeAction::FocusUrlBar);
        }

        self.url_bar_focused = false;
        None
    }

    pub fn handle_text_input(&mut self, text: &str) {
        if self.url_bar_focused {
            self.url_text.push_str(text);
        }
    }

    pub fn handle_key(&mut self, keycode: Keycode) -> Option<ChromeAction> {
        if !self.url_bar_focused {
            return None;
        }

        match keycode {
            Keycode::Return => {
                let url = self.url_text.clone();
                Some(ChromeAction::Navigate(url))
            }
            Keycode::Backspace => {
                self.url_text.pop();
                None
            }
            _ => None,
        }
    }

    pub fn build_display_list(&self) -> DisplayList {
        let mut list = DisplayList::new();

        // Chrome background
        list.push_rect(
            Rect::new(0.0, 0.0, self.width, self.height),
            Color::rgb(240, 240, 240),
        );

        // Back button
        list.push_rect(self.back_button, Color::rgb(200, 200, 200));
        list.push_text(
            "<".to_string(),
            self.back_button.x + 10.0,
            self.back_button.y + 5.0,
            Color::BLACK,
            16.0,
        );

        // Forward button
        list.push_rect(self.forward_button, Color::rgb(200, 200, 200));
        list.push_text(
            ">".to_string(),
            self.forward_button.x + 10.0,
            self.forward_button.y + 5.0,
            Color::BLACK,
            16.0,
        );

        // Stop/Reload button
        if self.loading {
            // Red X for stop
            list.push_rect(self.stop_button, Color::rgb(220, 180, 180));
            list.push_text(
                "X".to_string(),
                self.stop_button.x + 10.0,
                self.stop_button.y + 5.0,
                Color::rgb(180, 60, 60),
                16.0,
            );
        } else {
            // Gray R for reload
            list.push_rect(self.stop_button, Color::rgb(200, 200, 200));
            list.push_text(
                "R".to_string(),
                self.stop_button.x + 10.0,
                self.stop_button.y + 5.0,
                Color::rgb(80, 80, 80),
                16.0,
            );
        }

        // URL bar
        let url_bar_color = if self.url_bar_focused {
            Color::WHITE
        } else {
            Color::rgb(250, 250, 250)
        };
        list.push_rect(self.url_bar, url_bar_color);

        // Border - blue when loading, gray otherwise
        let border_color = if self.loading {
            Color::rgb(66, 133, 244) // Blue border when loading
        } else {
            Color::rgb(180, 180, 180)
        };
        list.push_border(self.url_bar, border_color, 1.0);

        // URL text
        let display_text = if self.url_text.is_empty() && self.url_bar_focused {
            "Enter URL...".to_string()
        } else {
            self.url_text.clone()
        };

        list.push_text(
            display_text,
            self.url_bar.x + 8.0,
            self.url_bar.y + 7.0,
            if self.url_text.is_empty() {
                Color::rgb(150, 150, 150)
            } else {
                Color::BLACK
            },
            14.0,
        );

        // Progress bar at bottom of URL bar
        if let Some(progress) = self.progress {
            let progress_height = 4.0;
            let progress_width = self.url_bar.width * progress;
            list.push_rect(
                Rect::new(
                    self.url_bar.x,
                    self.url_bar.y + self.url_bar.height - progress_height,
                    progress_width,
                    progress_height,
                ),
                Color::rgb(66, 133, 244), // Blue progress bar
            );
        }

        // JS toggle button
        let js_color = if self.js_enabled {
            Color::rgb(76, 175, 80) // Green when enabled
        } else {
            Color::rgb(158, 158, 158) // Gray when disabled
        };
        list.push_rect(self.js_toggle, js_color);
        list.push_text(
            "JS".to_string(),
            self.js_toggle.x + 5.0,
            self.js_toggle.y + 7.0,
            Color::WHITE,
            12.0,
        );

        // CSS toggle button
        let css_color = if self.css_enabled {
            Color::rgb(33, 150, 243) // Blue when enabled
        } else {
            Color::rgb(158, 158, 158) // Gray when disabled
        };
        list.push_rect(self.css_toggle, css_color);
        list.push_text(
            "CSS".to_string(),
            self.css_toggle.x + 2.0,
            self.css_toggle.y + 7.0,
            Color::WHITE,
            11.0,
        );

        // FPS and render time display (to the left of JS toggle)
        let stats_x = self.js_toggle.x - 95.0;
        let stats_text = format!("{:.0} FPS | {:.1}ms", self.fps, self.render_time_ms);
        // Color-code based on performance: green if > 55fps, yellow if > 30fps, red otherwise
        let stats_color = if self.fps > 55.0 {
            Color::rgb(76, 175, 80) // Green
        } else if self.fps > 30.0 {
            Color::rgb(255, 193, 7) // Yellow/amber
        } else {
            Color::rgb(244, 67, 54) // Red
        };
        list.push_text(
            stats_text,
            stats_x,
            self.js_toggle.y + 9.0,
            stats_color,
            10.0,
        );

        // Bottom border
        list.push_rect(
            Rect::new(0.0, self.height - 1.0, self.width, 1.0),
            Color::rgb(200, 200, 200),
        );

        list
    }
}
