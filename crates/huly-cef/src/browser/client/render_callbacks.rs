use std::sync::{Arc, Mutex};

use crate::{state::SharedBrowserState, Framebuffer, TabMessage};
use cef_ui::{Browser, PaintElementType, Rect, RenderHandlerCallbacks, ScreenInfo};

impl Framebuffer {
    fn new(width: u32, height: u32, dpr: f64) -> Self {
        let dpr_width = width as f64 * dpr;
        let dpr_height = height as f64 * dpr;
        let size = dpr_width * dpr_height * 4.0;
        Self {
            width: dpr_width as u32,
            height: dpr_height as u32,
            dpr,
            data: vec![0; size as usize],
        }
    }

    fn copy_rect(&mut self, src: &[u8], src_stride: usize, src_rect: &Rect, dst_rect: &Rect) {
        let src_x = src_rect.x as usize;
        let src_y = src_rect.y as usize;
        let dst_x = dst_rect.x as usize;
        let dst_y = dst_rect.y as usize;
        let width = src_rect.width as usize;
        let height = src_rect.height as usize;
        let dst_stride = (self.width * 4) as usize;

        for row in 0..height {
            let src_start = (src_y + row) * src_stride + src_x * 4;
            let src_end = src_start + width * 4;
            let dst_start = (dst_y + row) * dst_stride + dst_x * 4;
            let dst_end = dst_start + width * 4;

            Self::convert_bgra_to_rgba(
                &mut self.data[dst_start..dst_end],
                &src[src_start..src_end],
            );
        }
    }

    fn convert_bgra_to_rgba(dst: &mut [u8], src: &[u8]) {
        for (src, dst) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
            let [b, g, r, a] = src.try_into().unwrap();
            dst.copy_from_slice(&[r, g, b, a]);
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct HulyRenderHandlerCallbacks {
    state: SharedBrowserState,

    framebuffer: Arc<Mutex<Framebuffer>>,
    popup_rect: Option<Rect>,
    popup_data: Option<Vec<u8>>,
}

impl HulyRenderHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        let (w, h, dpr) = state.read(|s| (s.width, s.height, s.dpr));
        let framebuffer = Arc::new(Mutex::new(Framebuffer::new(w, h, dpr)));

        Self {
            state,
            framebuffer,
            popup_rect: None,
            popup_data: None,
        }
    }

    fn draw_view(&mut self, buffer: &[u8], width: usize, dirty_rects: &[Rect]) {
        let mut framebuffer = self.framebuffer.lock().unwrap();
        if framebuffer.len() != buffer.len() {
            return;
        }

        let src_stride = width * 4;
        for rect in dirty_rects {
            framebuffer.copy_rect(buffer, src_stride, rect, rect);
        }
    }

    fn draw_popup(&mut self) {
        let mut framebuffer = self.framebuffer.lock().unwrap();

        let popup_rect = self
            .popup_rect
            .as_ref()
            .expect("popup rect can't be None here");

        let popup_data = self
            .popup_data
            .as_ref()
            .expect("popup data can't be None here");

        let src_stride = popup_rect.width as usize * 4;
        let src_rect = &Rect {
            x: 0,
            y: 0,
            width: popup_rect.width,
            height: popup_rect.height,
        };

        framebuffer.copy_rect(popup_data, src_stride, src_rect, popup_rect);
    }
}

impl RenderHandlerCallbacks for HulyRenderHandlerCallbacks {
    fn get_view_rect(&mut self, _: Browser) -> Rect {
        let (w, h, dpr) = self.state.read(|s| (s.width, s.height, s.dpr));
        let new_fb_length = Framebuffer::length_in_bytes(w, h, dpr);

        let mut framebuffer = self.framebuffer.lock().unwrap();
        if framebuffer.len() != new_fb_length {
            *framebuffer = Framebuffer::new(w, h, dpr);
        }

        Rect {
            x: 0,
            y: 0,
            width: w as i32,
            height: h as i32,
        }
    }

    fn get_screen_info(&mut self, _: Browser) -> Option<ScreenInfo> {
        let (w, h, dpr) = self
            .state
            .read(|s| (s.width as i32, s.height as i32, s.dpr));

        Some(ScreenInfo {
            device_scale_factor: dpr as f32,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect: Rect {
                x: 0,
                y: 0,
                width: w,
                height: h,
            },
            available_rect: Rect {
                x: 0,
                y: 0,
                width: w,
                height: h,
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
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: usize,
        _height: usize,
    ) {
        let active = self.state.read(|state| state.active);
        if !active {
            return;
        }

        match paint_element_type {
            PaintElementType::View => {
                self.draw_view(buffer, width, dirty_rects);
            }
            PaintElementType::Popup => {
                self.popup_data = Some(buffer.to_vec());
            }
        }

        if self.popup_rect.is_some() && self.popup_data.is_some() {
            self.draw_popup();
        }

        self.state
            .notify(TabMessage::Frame(self.framebuffer.clone()));
    }
}
