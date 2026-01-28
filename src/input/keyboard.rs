use super::events::Modifiers;

pub struct KeyboardState {
    pub modifiers: Modifiers,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            modifiers: Modifiers::default(),
        }
    }

    pub fn update_modifiers(&mut self, keymod: sdl2::keyboard::Mod) {
        self.modifiers.shift = keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
            || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD);
        self.modifiers.ctrl = keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
            || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD);
        self.modifiers.alt = keymod.contains(sdl2::keyboard::Mod::LALTMOD)
            || keymod.contains(sdl2::keyboard::Mod::RALTMOD);
        self.modifiers.meta = keymod.contains(sdl2::keyboard::Mod::LGUIMOD)
            || keymod.contains(sdl2::keyboard::Mod::RGUIMOD);
    }
}
