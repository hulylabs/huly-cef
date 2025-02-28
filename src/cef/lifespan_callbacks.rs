use cef_ui::{
    Browser, BrowserSettings, Client, DictionaryValue, Frame, LifeSpanHandlerCallbacks, PopupFeatures, WindowInfo, WindowOpenDisposition
};

pub struct MyLifeSpanHandlerCallbacks;

#[allow(unused_variables)]
impl LifeSpanHandlerCallbacks for MyLifeSpanHandlerCallbacks {
    unsafe fn on_before_popup(
        &mut self,
        _browser: Browser,
        frame: Frame,
        target_url: Option<String>,
        target_frame_name: Option<String>,
        target_disposition: WindowOpenDisposition,
        user_gesture: bool,
        popup_features: PopupFeatures,
        window_info: &mut WindowInfo,
        client: &mut Option<Client>,
        settings: &mut BrowserSettings,
        extra_info: &mut Option<DictionaryValue>,
        no_javascript_access: &mut bool
    ) -> bool {
        true
    }

    fn on_before_dev_tools_popup(
        &mut self,
        _browser: Browser,
        window_info: &mut WindowInfo,
        client: &mut Option<Client>,
        settings: &mut BrowserSettings,
        extra_info: &mut Option<DictionaryValue>,
        use_default_window: &mut bool
    ) {
    }
    
    fn on_after_created(&mut self, _browser: Browser) {
    }

    fn do_close(&mut self, _browser: Browser) -> bool {
        false
    }

    fn on_before_close(&mut self, _browser: Browser) {
        // If you have more than one _browser open, you want to only
        // call this when the number of open browsers reaches zero.
        // unsafe {
        //     cef_quit_message_loop();
        // }
    }
}


