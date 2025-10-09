use crate::{state::SharedBrowserState, MouseButton};
use cef_ui::{Browser, MouseButtonType, MouseEvent};
use log::info;

pub struct Mouse {
    inner: Browser,
    state: SharedBrowserState,
}

impl Clone for Mouse {
    fn clone(&self) -> Self {
        Mouse {
            inner: self.inner.clone(),
            state: self.state.clone(),
        }
    }
}

impl Mouse {
    pub fn new(inner: Browser, state: SharedBrowserState) -> Self {
        Mouse { inner, state }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        let modifiers = self.state.read(|s| s.input.get_event_flags());
        let event = MouseEvent { x, y, modifiers };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_move_event(&event, false)
            .expect("failed to send mouse move event");
    }

    pub fn click(&self, x: i32, y: i32, button: MouseButton, down: bool) {
        if button == MouseButton::Left {
            self.state.update(|state| {
                state.input.left_mouse_button_down = down;
            });
        }

        let modifiers = self.state.read(|s| s.input.get_event_flags());
        let event = MouseEvent { x, y, modifiers };

        let button = match button {
            MouseButton::Left => MouseButtonType::Left,
            MouseButton::Middle => MouseButtonType::Middle,
            MouseButton::Right => MouseButtonType::Right,
        };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_click_event(&event, button, !down, 1)
            .expect("failed to send mouse click event");
    }

    pub fn wheel(&self, x: i32, y: i32, dx: i32, dy: i32) {
        let modifiers = self.state.read(|s| s.input.get_event_flags());
        let event = MouseEvent { x, y, modifiers };
        self.inner
            .get_host()
            .unwrap()
            .send_mouse_wheel_event(&event, dx, dy)
            .expect("failed to send mouse wheel event");
    }
}
