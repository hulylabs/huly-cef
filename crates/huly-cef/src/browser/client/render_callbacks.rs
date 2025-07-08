use cef_ui::{
    AccessibilityHandler, Browser, DragData, DragOperations, HorizontalAlignment, PaintElementType,
    Point, Range, Rect, RenderHandlerCallbacks, ScreenInfo, Size, TextInputMode, TouchHandleState,
};

use crate::{state::SharedBrowserState, TabMessage};

pub struct HulyRenderHandlerCallbacks {
    state: SharedBrowserState,

    popup_rect: Option<Rect>,
    popup_data: Option<Vec<u8>>,
}

impl HulyRenderHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            popup_rect: None,
            popup_data: None,
        }
    }

    fn send_popup(&self) {
        if let (Some(rect), Some(data)) = (&self.popup_rect, &self.popup_data) {
            self.state.notify(TabMessage::Popup {
                x: rect.x,
                y: rect.y,
                width: rect.width as u32,
                height: rect.height as u32,
                data: data.clone(),
            });
        }
    }

    fn convert_bgra_to_rgba(&self, buffer: &[u8], width: usize, height: usize) -> Vec<u8> {
        let pixel_count = width * height;
        let mut rgba_buffer = vec![0u8; pixel_count * 4];
        for (src, dst) in buffer.chunks_exact(4).zip(rgba_buffer.chunks_exact_mut(4)) {
            let [b, g, r, a] = src.try_into().unwrap();
            dst.copy_from_slice(&[r, g, b, a]);
        }

        rgba_buffer
    }

    fn try_send_screenshot(&self, buffer: &[u8], width: u32, height: u32) -> bool {
        let mut state = self.state.lock();

        if state.screenshot_channel.is_some()
            && width == state.screenshot_width
            && height == state.screenshot_height
        {
            let tx = state.screenshot_channel.take().unwrap();
            _ = tx.send(buffer.to_vec());
            return true;
        }

        false
    }
}

impl RenderHandlerCallbacks for HulyRenderHandlerCallbacks {
    fn get_accessibility_handler(&mut self) -> Option<AccessibilityHandler> {
        None
    }

    fn get_root_screen_rect(&mut self, _: Browser) -> Option<Rect> {
        None
    }

    fn get_view_rect(&mut self, _: Browser) -> Rect {
        let state = self.state.lock();
        let mut rect = Rect {
            x: 0,
            y: 0,
            width: state.width as i32,
            height: state.height as i32,
        };

        if state.screenshot_channel.is_some() {
            rect.width = state.screenshot_width as i32;
            rect.height = state.screenshot_height as i32;
        }

        rect
    }

    fn get_screen_point(&mut self, _: Browser, _: &Point) -> Option<Point> {
        None
    }

    fn get_screen_info(&mut self, _: Browser) -> Option<ScreenInfo> {
        let state = self.state.lock();

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

    fn on_popup_show(&mut self, _: Browser, show: bool) {
        if !show {
            self.popup_rect = None;
            self.popup_data = None;
        }
    }

    fn on_popup_size(&mut self, _: Browser, rect: &Rect) {
        self.popup_rect = Some(rect.clone());
    }

    fn on_paint(
        &mut self,
        _: Browser,
        paint_element_type: PaintElementType,
        _: &[Rect],
        buffer: &[u8],
        width: usize,
        height: usize,
    ) {
        if self.try_send_screenshot(buffer, width as u32, height as u32) {
            return;
        }

        {
            let state = self.state.lock();
            if !(state.active && state.width == width as u32 && state.height == height as u32) {
                return;
            }
        }

        match paint_element_type {
            PaintElementType::Popup => {
                self.popup_data = Some(buffer.to_vec());
                self.send_popup();
            }
            PaintElementType::View => {
                self.state.notify(TabMessage::Frame(
                    self.convert_bgra_to_rgba(buffer, width, height),
                ));
                self.send_popup();
            }
        }
    }

    fn on_accelerated_paint(&mut self, _: Browser, _: PaintElementType, _: &[Rect]) {}

    fn get_touch_handle_size(&mut self, _: Browser, _: HorizontalAlignment) -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }

    fn on_touch_handle_state_changed(&mut self, _: Browser, _: &TouchHandleState) {}

    fn start_dragging(&mut self, _: Browser, _: DragData, _: DragOperations, _: &Point) -> bool {
        false
    }

    fn update_drag_cursor(&mut self, _: Browser, _: DragOperations) {}

    fn on_scroll_offset_changed(&mut self, _: Browser, _x: f64, _y: f64) {}

    fn on_ime_composition_range_changed(&mut self, _: Browser, _: &Range, _: &[Rect]) {}

    fn on_text_selection_changed(&mut self, _: Browser, _: Option<String>, _: &Range) {}

    fn on_virtual_keyboard_requested(&mut self, _: Browser, _: TextInputMode) {}
}
