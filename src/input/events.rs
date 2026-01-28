#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseDown { x: i32, y: i32, button: MouseButton },
    MouseUp { x: i32, y: i32, button: MouseButton },
    MouseMove { x: i32, y: i32 },
    MouseWheel { x: i32, y: i32 },
    KeyDown { keycode: KeyCode, modifiers: Modifiers },
    KeyUp { keycode: KeyCode, modifiers: Modifiers },
    TextInput { text: String },
    WindowResize { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Backspace,
    Tab,
    Return,
    Escape,
    Space,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl From<sdl2::mouse::MouseButton> for MouseButton {
    fn from(button: sdl2::mouse::MouseButton) -> Self {
        match button {
            sdl2::mouse::MouseButton::Left => MouseButton::Left,
            sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
            sdl2::mouse::MouseButton::Right => MouseButton::Right,
            _ => MouseButton::Left,
        }
    }
}

impl From<sdl2::keyboard::Keycode> for KeyCode {
    fn from(keycode: sdl2::keyboard::Keycode) -> Self {
        use sdl2::keyboard::Keycode as K;
        match keycode {
            K::Backspace => KeyCode::Backspace,
            K::Tab => KeyCode::Tab,
            K::Return => KeyCode::Return,
            K::Escape => KeyCode::Escape,
            K::Space => KeyCode::Space,
            K::Left => KeyCode::Left,
            K::Right => KeyCode::Right,
            K::Up => KeyCode::Up,
            K::Down => KeyCode::Down,
            K::Home => KeyCode::Home,
            K::End => KeyCode::End,
            K::PageUp => KeyCode::PageUp,
            K::PageDown => KeyCode::PageDown,
            K::Delete => KeyCode::Delete,
            K::A => KeyCode::A,
            K::B => KeyCode::B,
            K::C => KeyCode::C,
            K::D => KeyCode::D,
            K::E => KeyCode::E,
            K::F => KeyCode::F,
            K::G => KeyCode::G,
            K::H => KeyCode::H,
            K::I => KeyCode::I,
            K::J => KeyCode::J,
            K::K => KeyCode::K,
            K::L => KeyCode::L,
            K::M => KeyCode::M,
            K::N => KeyCode::N,
            K::O => KeyCode::O,
            K::P => KeyCode::P,
            K::Q => KeyCode::Q,
            K::R => KeyCode::R,
            K::S => KeyCode::S,
            K::T => KeyCode::T,
            K::U => KeyCode::U,
            K::V => KeyCode::V,
            K::W => KeyCode::W,
            K::X => KeyCode::X,
            K::Y => KeyCode::Y,
            K::Z => KeyCode::Z,
            K::Num0 => KeyCode::Num0,
            K::Num1 => KeyCode::Num1,
            K::Num2 => KeyCode::Num2,
            K::Num3 => KeyCode::Num3,
            K::Num4 => KeyCode::Num4,
            K::Num5 => KeyCode::Num5,
            K::Num6 => KeyCode::Num6,
            K::Num7 => KeyCode::Num7,
            K::Num8 => KeyCode::Num8,
            K::Num9 => KeyCode::Num9,
            K::F1 => KeyCode::F1,
            K::F2 => KeyCode::F2,
            K::F3 => KeyCode::F3,
            K::F4 => KeyCode::F4,
            K::F5 => KeyCode::F5,
            K::F6 => KeyCode::F6,
            K::F7 => KeyCode::F7,
            K::F8 => KeyCode::F8,
            K::F9 => KeyCode::F9,
            K::F10 => KeyCode::F10,
            K::F11 => KeyCode::F11,
            K::F12 => KeyCode::F12,
            _ => KeyCode::Unknown,
        }
    }
}
