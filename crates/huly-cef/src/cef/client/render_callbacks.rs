use std::sync::{Arc, Mutex};

use cef_ui::{
    AccessibilityHandler, Browser, DragData, DragOperations, HorizontalAlignment, PaintElementType,
    Point, Range, Rect, RenderHandlerCallbacks, ScreenInfo, Size, TextInputMode, TouchHandleState,
};
use log;
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::{browser::BrowserState, messages::CefMessage};

pub struct HulyRenderHandlerCallbacks {
    cef_msg_channel: UnboundedSender<CefMessage>,
    browser_state: Arc<Mutex<BrowserState>>,
}

impl HulyRenderHandlerCallbacks {
    pub fn new(
        cef_msg_channel: UnboundedSender<CefMessage>,
        browser_state: Arc<Mutex<BrowserState>>,
    ) -> Self {
        Self {
            cef_msg_channel,
            browser_state,
        }
    }
}

impl RenderHandlerCallbacks for HulyRenderHandlerCallbacks {
    fn get_accessibility_handler(&mut self) -> Option<AccessibilityHandler> {
        None
    }

    fn get_root_screen_rect(&mut self, _browser: Browser) -> Option<Rect> {
        let state = self.browser_state.lock().unwrap();
        Some(Rect {
            x: 0,
            y: 0,
            width: state.width as i32,
            height: state.height as i32,
        })
    }

    fn get_view_rect(&mut self, _browser: Browser) -> Rect {
        let state = self.browser_state.lock().unwrap();
        Rect {
            x: 0,
            y: 0,
            width: state.width as i32,
            height: state.height as i32,
        }
    }

    fn get_screen_point(&mut self, _browser: Browser, _view: &Point) -> Option<Point> {
        Some(Point { x: 0, y: 0 })
    }

    fn get_screen_info(&mut self, _browser: Browser) -> Option<ScreenInfo> {
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
        _paint_element_type: PaintElementType,
        _dirty_rects: &[Rect],
        buffer: &[u8],
        width: usize,
        height: usize,
    ) {
        let state = self.browser_state.lock().unwrap();
        if state.width != width as u32 || state.height != height as u32 {
            return;
        }

        if state.active {
            let pixel_count = width * height * 4;
            let mut rgba_buffer = vec![0u8; pixel_count];
            for (src, dst) in buffer.chunks_exact(4).zip(rgba_buffer.chunks_exact_mut(4)) {
                let [b, g, r, a] = src.try_into().unwrap();
                dst.copy_from_slice(&[r, g, b, a]);
            }

            if let Err(error) = self.cef_msg_channel.send(CefMessage::Frame(rgba_buffer)) {
                log::error!("Failed to send message: {:?}", error);
            }
        }
    }

    fn on_accelerated_paint(
        &mut self,
        _browser: Browser,
        _paint_element_type: PaintElementType,
        _dirty_rects: &[Rect],
    ) {
    }

    fn get_touch_handle_size(
        &mut self,
        _browser: Browser,
        _orientation: HorizontalAlignment,
    ) -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }

    fn on_touch_handle_state_changed(&mut self, _browser: Browser, _state: &TouchHandleState) {}

    fn start_dragging(
        &mut self,
        _browser: Browser,
        _drag_data: DragData,
        _allowed_ops: DragOperations,
        _drag_start: &Point,
    ) -> bool {
        false
    }

    fn update_drag_cursor(&mut self, _browser: Browser, _operation: DragOperations) {}

    fn on_scroll_offset_changed(&mut self, _browser: Browser, _x: f64, _y: f64) {}

    fn on_ime_composition_range_changed(
        &mut self,
        _browser: Browser,
        _selected_range: &Range,
        _character_bounds: &[Rect],
    ) {
    }

    fn on_text_selection_changed(
        &mut self,
        _browser: Browser,
        _selected_text: Option<String>,
        _selected_range: &Range,
    ) {
    }

    fn on_virtual_keyboard_requested(&mut self, _browser: Browser, _input_mode: TextInputMode) {}
}
