pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_down: bool,
    pub middle_down: bool,
    pub right_down: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            left_down: false,
            middle_down: false,
            right_down: false,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_button(&mut self, button: super::events::MouseButton, down: bool) {
        match button {
            super::events::MouseButton::Left => self.left_down = down,
            super::events::MouseButton::Middle => self.middle_down = down,
            super::events::MouseButton::Right => self.right_down = down,
        }
    }
}
