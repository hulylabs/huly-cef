use std::sync::{Arc, Mutex};

use cef_ui::{Browser, Point, Rect, RenderHandlerCallbacks, ScreenInfo, Size};

use super::BrowserState;

pub struct MyRenderHandlerCallbacks {
    browser_state: Arc<Mutex<BrowserState>>,
}

impl MyRenderHandlerCallbacks {
    pub fn new(browser_state: Arc<Mutex<BrowserState>>) -> Self {
        Self {
            browser_state: browser_state,
        }
    }
}

impl RenderHandlerCallbacks for MyRenderHandlerCallbacks {
    fn get_accessibility_handler(&mut self) -> Option<cef_ui::AccessibilityHandler> {
        None
    }

    fn get_root_screen_rect(&mut self, _browser: Browser) -> Option<Rect> {
        let state = self.browser_state.lock().unwrap();
        Some(cef_ui::Rect {
            x: 0,
            y: 0,
            width: state.width as i32,
            height: state.height as i32,
        })
    }

    fn get_view_rect(&mut self, _browser: Browser) -> Rect {
        let state = self.browser_state.lock().unwrap();
        cef_ui::Rect {
            x: 0,
            y: 0,
            width: state.width as i32,
            height: state.height as i32,
        }
    }

    fn get_screen_point(&mut self, _browser: Browser, _view: &Point) -> Option<Point> {
        Some(Point { x: 0, y: 0 })
    }

    fn get_screen_info(&mut self, _browser: Browser) -> Option<cef_ui::ScreenInfo> {
        let state = self.browser_state.lock().unwrap();

        Some(ScreenInfo {
            device_scale_factor: 1.0,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect: Rect {
                x: 0,
                y: 0,
                width: state.width as i32,
                height: state.height as i32,
            },
            available_rect: Rect {
                x: 0,
                y: 0,
                width: state.width as i32,
                height: state.height as i32,
            },
        })
    }

    fn on_popup_show(&mut self, _browser: Browser, _show: bool) {}

    fn on_popup_size(&mut self, _browser: Browser, _rect: &Rect) {}

    fn on_paint(
        &mut self,
        _browser: Browser,
        _paint_element_type: cef_ui::PaintElementType,
        _dirty_rects: &[Rect],
        buffer: &[u8],
        _width: usize,
        _height: usize,
    ) {
        let now = std::time::Instant::now();
        let state = self.browser_state.lock().unwrap();
        state.tx.send(buffer.to_vec()).unwrap();
        println!("on_paint: {:?}", now.elapsed());
    }

    fn on_accelerated_paint(
        &mut self,
        _browser: Browser,
        _paint_element_type: cef_ui::PaintElementType,
        _dirty_rects: &[Rect],
        _shared_handle: *mut std::ffi::c_void,
    ) {
    }

    fn get_touch_handle_size(
        &mut self,
        _browser: Browser,
        _orientation: cef_ui::HorizontalAlignment,
    ) -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }

    fn on_touch_handle_state_changed(
        &mut self,
        _browser: Browser,
        _state: &cef_ui::TouchHandleState,
    ) {
    }

    fn start_dragging(
        &mut self,
        _browser: Browser,
        _drag_data: cef_ui::DragData,
        _allowed_ops: cef_ui::DragOperations,
        _drag_start: &Point,
    ) -> bool {
        false
    }

    fn update_drag_cursor(&mut self, _browser: Browser, _operation: cef_ui::DragOperations) {}

    fn on_scroll_offset_changed(&mut self, _browser: Browser, _x: f64, _y: f64) {}

    fn on_ime_composition_range_changed(
        &mut self,
        _browser: Browser,
        _selected_range: &cef_ui::Range,
        _character_bounds: &[Rect],
    ) {
    }

    fn on_text_selection_changed(
        &mut self,
        _browser: Browser,
        _selected_text: Option<String>,
        _selected_range: &cef_ui::Range,
    ) {
    }

    fn on_virtual_keyboard_requested(
        &mut self,
        _browser: Browser,
        _input_mode: cef_ui::TextInputMode,
    ) {
    }
}
