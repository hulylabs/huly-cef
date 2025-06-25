use log::error;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, EventFlags, KeyEvent, KeyEventType,
    MouseButtonType, MouseEvent, PaintElementType, StringVisitor, StringVisitorCallbacks, ThreadId,
    WindowInfo,
};
use crossbeam_channel::Sender;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{
    client,
    messages::{LoadStatus, MouseType, TabMessage},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickableElement {
    pub tag: String,
    pub text: String,
    pub x: i32,
    pub y: i32,
}

/// Maintains the state of a browser instance.
pub struct BrowserState {
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub load_status: LoadStatus,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub error_code: i32,
    pub error_text: String,
    pub cursor: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
    pub left_mouse_button_down: bool,

    pub clickable_elements: Option<Vec<ClickableElement>>,
    pub clickable_elements_channel: Option<oneshot::Sender<Vec<ClickableElement>>>,

    pub screenshot_width: u32,
    pub screenshot_height: u32,
    pub screenshot_channel: Option<oneshot::Sender<Vec<u8>>>,
}

pub struct Browser {
    inner: cef_ui::Browser,
    pub state: Arc<Mutex<BrowserState>>,
}

impl Clone for Browser {
    fn clone(&self) -> Self {
        Browser {
            inner: self.inner.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Browser {
    pub fn new(width: u32, height: u32, url: &str, sender: UnboundedSender<TabMessage>) -> Self {
        create_browser(width, height, url, sender)
    }

    pub fn mouse_move(&self, x: i32, y: i32) {
        let mut modifiers = EventFlags::empty();
        if self.state.lock().unwrap().left_mouse_button_down {
            modifiers.insert(EventFlags::LeftMouseButton);
        }

        let event = MouseEvent { x, y, modifiers };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_move_event(&event, false)
            .expect("failed to send mouse move event");
    }

    pub fn mouse_click(&self, x: i32, y: i32, button: MouseType, down: bool) {
        if button == MouseType::Left {
            let mut state = self.state.lock().unwrap();
            state.left_mouse_button_down = down;
        }

        let event = MouseEvent {
            x,
            y,
            modifiers: EventFlags::empty(),
        };

        let button = match button {
            MouseType::Left => MouseButtonType::Left,
            MouseType::Middle => MouseButtonType::Middle,
            MouseType::Right => MouseButtonType::Right,
        };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_click_event(&event, button, !down, 1)
            .expect("failed to send mouse click event");
    }

    pub fn mouse_wheel(&self, x: i32, y: i32, dx: i32, dy: i32) {
        let event = MouseEvent {
            x,
            y,
            modifiers: EventFlags::empty(),
        };
        self.inner
            .get_host()
            .unwrap()
            .send_mouse_wheel_event(&event, dx, -dy)
            .expect("failed to send mouse wheel event");
    }

    pub fn key_press(
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
        let mut event = KeyEvent {
            event_type,
            modifiers,
            windows_key_code: windowscode.into(),
            native_key_code: code,
            is_system_key: false,
            character,
            unmodified_character: character,
            focus_on_editable_field: false,
        };

        _ = self.inner.get_host().unwrap().send_key_event(event.clone());

        if event_type == KeyEventType::KeyDown && character != 0 {
            event.event_type = KeyEventType::Char;
            _ = self.inner.get_host().unwrap().send_key_event(event);
        }
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

    pub fn start_video(&self) {
        let mut state = self.state.lock().unwrap();
        state.active = true;

        _ = self.inner.get_host().unwrap().was_hidden(false);
        _ = self.inner.get_host().unwrap().set_focus(true);

        _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn stop_video(&self) {
        let mut state = self.state.lock().unwrap();
        state.active = false;

        _ = self.inner.get_host().unwrap().was_hidden(true);
    }

    pub fn resize(&self, width: u32, height: u32) {
        let mut state = self.state.lock().unwrap();
        state.width = width;
        state.height = height;

        let _ = self.inner.get_host().unwrap().was_resized();
        let _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn go_to(&self, url: &str) {
        let _ = self.inner.get_main_frame().unwrap().unwrap().load_url(url);
    }

    pub fn go_back(&self) {
        let _ = self.inner.go_back();
    }

    pub fn go_forward(&self) {
        let _ = self.inner.go_forward();
    }

    pub fn reload(&self) {
        let _ = self.inner.reload();
    }

    pub fn close(&self) {
        let _ = self.inner.get_host().unwrap().close_browser(true);
    }

    pub fn get_id(&self) -> i32 {
        self.inner
            .get_identifier()
            .expect("failed to get browser ID")
    }

    pub fn set_focus(&self, focus: bool) {
        let _ = self.inner.get_host().unwrap().set_focus(focus);
    }

    pub async fn get_dom(&self) -> String {
        let (tx, rx) = oneshot::channel::<String>();
        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .get_source(StringVisitor::new(DOMVisitor::new(tx)));

        rx.await.unwrap()
    }

    pub fn set_text(&self, selector: &str, text: &str) {
        let code = format!("document.querySelector('{}').value = '{}';", selector, text);
        log::info!("Executing JavaScript to set text: {}", code);
        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .execute_java_script(&code, "", 0);
    }

    pub async fn screenshot(&self, width: u32, height: u32) -> Vec<u8> {
        let (tx, rx) = oneshot::channel::<Vec<u8>>();

        {
            let mut state = self.state.lock().unwrap();
            state.screenshot_width = width;
            state.screenshot_height = height;
            state.screenshot_channel = Some(tx);
        }

        _ = self.inner.get_host().unwrap().was_resized();
        rx.await.unwrap()
    }

    pub async fn get_clickable_elements(&self) -> Vec<ClickableElement> {
        let (tx, rx) = oneshot::channel::<Vec<ClickableElement>>();
        {
            let mut state = self.state.lock().unwrap();
            state.clickable_elements_channel = Some(tx);
        }
        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .execute_java_script(GET_CLICKABLE_ELEMENTS_JS, "", 0);

        let clickable_elements = rx.await.unwrap();
        {
            let mut state = self.state.lock().unwrap();
            state.clickable_elements = Some(clickable_elements.clone());
        }

        clickable_elements
    }
}

struct DOMVisitor {
    tx: Option<oneshot::Sender<String>>,
}

impl DOMVisitor {
    pub fn new(tx: oneshot::Sender<String>) -> Self {
        DOMVisitor { tx: Some(tx) }
    }
}

impl StringVisitorCallbacks for DOMVisitor {
    fn visit(&mut self, string: &str) {
        if let Err(e) = self.tx.take().unwrap().send(string.to_string()) {
            error!("failed to get DOM: {}", e);
        }
    }
}

struct CreateBrowserTaskCallback {
    tx: Sender<Browser>,
    width: u32,
    height: u32,
    url: String,
    event_channel: UnboundedSender<TabMessage>,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    /// Executes the task to create a browser and send it through the channel.
    fn execute(&mut self) {
        let window_info = WindowInfo::new().windowless_rendering_enabled(true);
        let settings = BrowserSettings::new().windowless_frame_rate(60);
        let state = Arc::new(Mutex::new(BrowserState {
            title: "".to_string(),
            url: self.url.clone(),
            favicon: None,
            load_status: LoadStatus::Loading,
            can_go_back: false,
            can_go_forward: false,
            error_code: 0,
            error_text: "".to_string(),
            cursor: "Pointer".to_string(),
            width: self.width,
            height: self.height,
            active: true,
            left_mouse_button_down: false,
            clickable_elements: None,
            clickable_elements_channel: None,
            screenshot_width: 0,
            screenshot_height: 0,
            screenshot_channel: None,
        }));

        let client = client::new(state.clone(), self.event_channel.clone());
        let inner = BrowserHost::create_browser_sync(
            &window_info,
            client,
            &self.url,
            &settings,
            None,
            None,
        );

        self.tx
            .send(Browser { inner, state })
            .expect("failed to send a browser");
    }
}

/// Creates a new browser instance.
///
/// # Parameters
///
/// - `width`: The width of the browser.
/// - `height`: The height of the browser.
/// - `url`: The URL to load in the browser.
/// - `sender`: A channel for CEF messages.
///
/// # Returns
///
/// A new instance of a CEF browser.
///
/// # Panics
///
/// This function will panic if it fails to create a browser in the UI thread.
fn create_browser(
    width: u32,
    height: u32,
    url: &str,
    event_channel: UnboundedSender<TabMessage>,
) -> Browser {
    let (tx, rx) = crossbeam_channel::unbounded::<Browser>();
    let result = cef_ui::post_task(
        ThreadId::UI,
        CefTask::new(CreateBrowserTaskCallback {
            tx,
            width,
            height,
            url: url.to_string(),
            event_channel,
        }),
    );

    if !result {
        panic!("failed to create a browser in the UI thread");
    }

    rx.recv()
        .expect("failed to receive a CEF browser, created in the UI thread")
}

const GET_CLICKABLE_ELEMENTS_JS: &str = r#"
    function isInteractiveElement(element) {
        // Immediately return false for body tag
        if (element.tagName && element.tagName.toLowerCase() === 'body') {
            return false;
        }

        // Base interactive elements and roles
        const interactiveElements = new Set([
            'a',
            'button',
            'details',
            'embed',
            'input',
            'label',
            'menu',
            'menuitem',
            'object',
            'select',
            'textarea',
            'summary',
        ]);

        const interactiveRoles = new Set([
            'button',
            'menu',
            'menuitem',
            'link',
            'checkbox',
            'radio',
            'slider',
            'tab',
            'tabpanel',
            'textbox',
            'combobox',
            'grid',
            'listbox',
            'option',
            'progressbar',
            'scrollbar',
            'searchbox',
            'switch',
            'tree',
            'treeitem',
            'spinbutton',
            'tooltip',
            'a-button-inner',
            'a-dropdown-button',
            'click',
            'menuitemcheckbox',
            'menuitemradio',
            'a-button-text',
            'button-text',
            'button-icon',
            'button-icon-only',
            'button-text-icon-only',
            'dropdown',
            'combobox',
        ]);

        const tagName = element.tagName ? element.tagName.toLowerCase() : null;
        const role = element.getAttribute('role');
        const ariaRole = element.getAttribute('aria-role');
        const tabIndex = element.getAttribute('tabindex');


        // Basic role/attribute checks
        const hasInteractiveRole =
            interactiveElements.has(tagName) ||
            interactiveRoles.has(role) ||
            interactiveRoles.has(ariaRole) ||
            (tabIndex !== null &&
                tabIndex !== '-1' &&
                element.parentElement?.tagName &&
                element.parentElement?.tagName.toLowerCase() !== 'body') ||
            element.getAttribute('data-action') === 'a-dropdown-select' ||
            element.getAttribute('data-action') === 'a-dropdown-button';

        if (hasInteractiveRole) return true;

        // Get computed style
        const style = getCachedComputedStyle(element);

        // Check for event listeners
        const hasClickHandler =
            element.onclick !== null ||
            element.getAttribute('onclick') !== null ||
            element.hasAttribute('ng-click') ||
            element.hasAttribute('@click') ||
            element.hasAttribute('v-on:click');

        // Helper function to safely get event listeners
        function getEventListeners(el) {
            try {
                // Try to get listeners using Chrome DevTools API
                return window.getEventListeners?.(el) || {};
            } catch (e) {
                // Fallback: check for common event properties
                const listeners = {};

                // List of common event types to check
                const eventTypes = [
                    'click',
                    'mousedown',
                    'mouseup',
                    'touchstart',
                    'touchend',
                    'keydown',
                    'keyup',
                    'focus',
                    'blur',
                ];

                for (const type of eventTypes) {
                    const handler = el[`on${type}`];
                    if (handler) {
                        listeners[type] = [
                            {
                                listener: handler,
                                useCapture: false,
                            },
                        ];
                    }
                }

                return listeners;
            }
        }

        // Check for click-related events on the element itself
        const listeners = getEventListeners(element);
        const hasClickListeners =
            listeners &&
            (listeners.click?.length > 0 ||
                listeners.mousedown?.length > 0 ||
                listeners.mouseup?.length > 0 ||
                listeners.touchstart?.length > 0 ||
                listeners.touchend?.length > 0);

        // Check for ARIA properties that suggest interactivity
        const hasAriaProps =
            element.hasAttribute('aria-expanded') ||
            element.hasAttribute('aria-pressed') ||
            element.hasAttribute('aria-selected') ||
            element.hasAttribute('aria-checked');

        // Check for form-related functionality
        const isFormRelated =
            element.form !== undefined ||
            element.hasAttribute('contenteditable') ||
            style.userSelect !== 'none';

        // Check if element is draggable
        const isDraggable =
            element.draggable || element.getAttribute('draggable') === 'true';

        // Additional check to prevent body from being marked as interactive
        if (
            (element.tagName && element.tagName.toLowerCase() === 'body') ||
            (element.parentElement &&
                element.parentElement.tagName &&
                element.parentElement.tagName.toLowerCase() === 'body')
        ) {
            return false;
        }

        return (
            hasAriaProps ||
            // hasClickStyling ||
            hasClickHandler ||
            hasClickListeners ||
            // isFormRelated ||
            isDraggable
        );
    }

    function isElementVisible(element) {
        const style = getCachedComputedStyle(element);
        return (
            element.offsetWidth > 0 &&
            element.offsetHeight > 0 &&
            style.visibility !== 'hidden' &&
            style.display !== 'none'
        );
    }

    // Cache for computed styles to improve performance
    const computedStyleCache = new WeakMap();

    function getCachedComputedStyle(element) {
        if (!computedStyleCache.has(element)) {
            computedStyleCache.set(element, window.getComputedStyle(element));
        }
        return computedStyleCache.get(element);
    }

    // Function to find all clickable elements and their centers
    const clickableElements = [];
    const processedElements = new Set();

    function walkDOM(node) {
        // Skip text nodes and other non-element nodes
        if (node.nodeType !== Node.ELEMENT_NODE) {
            return;
        }

        const element = node;

        // Skip if already processed
        if (processedElements.has(element)) {
            return;
        }

        processedElements.add(element);

        // Check if element is interactive and visible
        if (isInteractiveElement(element) && isElementVisible(element)) {
            const rect = element.getBoundingClientRect();
            const centerX = rect.left + rect.width / 2;
            const centerY = rect.top + rect.height / 2;

            let innerText = element.getAttribute('aria-label') || element.innerText || element.textContent || '';
            innerText = innerText.trim();

            if (element.tagName === 'INPUT') {
                if (element.type === 'submit' || element.type === 'button' || element.type === 'reset') {
                    innerText = element.value || innerText;
                } else if (element.type === 'checkbox' || element.type === 'radio') {
                    innerText = element.nextElementSibling?.innerText ||
                        document.querySelector(`label[for="${element.id}"]`)?.innerText ||
                        `${element.type}:${element.value || element.id}`;
                }
            }

            // For select elements, show current selection or placeholder
            if (element.tagName === 'SELECT') {
                innerText = element.options[element.selectedIndex]?.text || 'Select dropdown';
            }

            // For images, use alt text
            if (element.tagName === 'IMG') {
                innerText = element.alt || 'Image';
            }

            // Limit text length for display
            if (innerText.length > 50) {
                innerText = innerText.substring(0, 47) + '...';
            }

            clickableElements.push({
                element: element,
                tag: element.tagName.toLowerCase(),
                text: innerText,
                x: Math.round(centerX),
                y: Math.round(centerY),
            });
        }

        // Recursively walk child nodes
        for (let i = 0; i < element.children.length; i++) {
            walkDOM(element.children[i]);
        }
    }

    // Start walking from document.body
    walkDOM(document.body);

    getClickableElements(clickableElements);
    "#;
