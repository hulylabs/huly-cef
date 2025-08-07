use cef_ui::{Browser, EventFlags, KeyEvent, KeyEventType};

use crate::state::SharedBrowserState;

pub struct Keyboard {
    inner: Browser,
    state: SharedBrowserState,
}

impl Clone for Keyboard {
    fn clone(&self) -> Self {
        Keyboard {
            inner: self.inner.clone(),
            state: self.state.clone(),
        }
    }
}

impl Keyboard {
    pub fn new(inner: Browser, state: SharedBrowserState) -> Self {
        Keyboard { inner, state }
    }

    pub fn key(
        &self,
        character: u16,
        windowscode: i32,
        code: i32,
        down: bool,
        ctrl: bool,
        shift: bool,
    ) {
        let mut event_type = KeyEventType::KeyUp;
        if down {
            event_type = KeyEventType::KeyDown;
        };

        let mut modifiers = EventFlags::empty();
        if ctrl {
            modifiers = modifiers.union(EventFlags::ControlDown);
        }
        if shift {
            modifiers = modifiers.union(EventFlags::ShiftDown);
        }
        let event = KeyEvent {
            event_type,
            modifiers,
            windows_key_code: windowscode.into(),
            native_key_code: code,
            is_system_key: false,
            character,
            unmodified_character: character,
            focus_on_editable_field: false,
        };

        if cfg!(target_os = "macos") && event.event_type == KeyEventType::KeyUp {
            return;
        }
        _ = self.inner.get_host().unwrap().send_key_event(event.clone());
    }

    pub fn char(&self, character: u16) {
        let event = KeyEvent {
            event_type: KeyEventType::Char,
            modifiers: EventFlags::empty(),
            windows_key_code: 0.into(),
            native_key_code: 0,
            is_system_key: false,
            character,
            unmodified_character: character,
            focus_on_editable_field: false,
        };

        _ = self.inner.get_host().unwrap().send_key_event(event);
    }
}
