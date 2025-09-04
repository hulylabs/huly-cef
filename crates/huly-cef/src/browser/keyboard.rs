use cef_ui::{Browser, BrowserHost, EventFlags, KeyEvent, KeyEventType};

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
        let host = self.inner.get_host().unwrap();
        process_key_event(&host, character, windowscode, code, down, ctrl, shift);
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn process_key_event(
    host: &BrowserHost,
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
        modifiers = modifiers | EventFlags::ControlDown;
    }
    if shift {
        modifiers = modifiers | EventFlags::ShiftDown;
    }

    let mut event = KeyEvent {
        event_type,
        modifiers,
        windows_key_code: windowscode.into(),
        native_key_code: code,
        is_system_key: false,
        character,
        unmodified_character: to_lower_case(character),
        focus_on_editable_field: false,
    };

    _ = host.send_key_event(event.clone());

    if event_type == KeyEventType::KeyDown {
        event.event_type = KeyEventType::Char;
        _ = host.send_key_event(event);
    }
}

#[cfg(target_os = "windows")]
fn process_key_event(
    host: &BrowserHost,
    character: u16,
    windowscode: i32,
    code: i32,
    down: bool,
    ctrl: bool,
    shift: bool,
) {
    let mut modifiers = EventFlags::empty();
    if ctrl {
        modifiers = modifiers | EventFlags::ControlDown;
    }
    if shift {
        modifiers = modifiers | EventFlags::ShiftDown;
    }

    if down {
        let event = KeyEvent {
            event_type: KeyEventType::RawKeyDown,
            modifiers,
            windows_key_code: windowscode.into(),
            native_key_code: code,
            is_system_key: false,
            character: 0,
            unmodified_character: 0,
            focus_on_editable_field: false,
        };

        _ = host.send_key_event(event);

        if character > 0 {
            let char_event = KeyEvent {
                event_type: KeyEventType::Char,
                modifiers,
                windows_key_code: (character as i32).into(),
                native_key_code: 0,
                is_system_key: false,
                character: 0,
                unmodified_character: 0,
                focus_on_editable_field: false,
            };
            _ = host.send_key_event(char_event);
        }
    } else {
        let up_event = KeyEvent {
            event_type: KeyEventType::KeyUp,
            modifiers,
            windows_key_code: windowscode.into(),
            native_key_code: code,
            is_system_key: false,
            character: 0,
            unmodified_character: 0,
            focus_on_editable_field: false,
        };
        _ = host.send_key_event(up_event);
    }
}

#[allow(dead_code)]
fn to_lower_case(unicode: u16) -> u16 {
    char::from_u32(unicode as u32)
        .expect("invalid unicode character")
        .to_lowercase()
        .next()
        .expect("invalid unicode character") as u16
}
