pub const INTERACTIVE_ELEMENT_FUNCTION: &str = r#"
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
    const style = window.getComputedStyle(element);

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
"#;

pub const IS_ELEMENT_VISIBLE_FUNCTION: &str = r#"
function isElementVisible(element) {
    const style = window.getComputedStyle(element);
    return (
        element.offsetWidth > 0 &&
        element.offsetHeight > 0 &&
        style.visibility !== 'hidden' &&
        style.display !== 'none'
    );
}
"#;

pub const WALK_DOM_FUNCTION: &str = r#"
function walkDOM(node, clickableElements, processedElements) {
    if (node.nodeType !== Node.ELEMENT_NODE) {
        return;
    }

    const element = node;
    if (processedElements.has(element)) {
        return;
    }
    processedElements.add(element);

    if (isInteractiveElement(element) && isElementVisible(element)) {
        let innerText = element.getAttribute('aria-label') || element.innerText || element.textContent || '';
        innerText = innerText.trim();

        if (element.tagName === 'INPUT') {
            if (element.type === 'text') {
                innerText = element.getAttribute('placeholder') || element.value || innerText;
            }
            if (element.type === 'submit' || element.type === 'button' || element.type === 'reset') {
                innerText = element.value || innerText;
            } else if (element.type === 'checkbox' || element.type === 'radio') {
                innerText = element.nextElementSibling?.innerText ||
                    document.querySelector(`label[for="${element.id}"]`)?.innerText ||
                    `${element.type}:${element.value || element.id}`;
            }
        }

        if (element.tagName === 'SELECT') {
            innerText = element.options[element.selectedIndex]?.text || 'Select dropdown';
        }

        if (element.tagName === 'IMG') {
            innerText = element.alt || 'Image';
        }

        if (innerText.length > 50) {
            innerText = innerText.substring(0, 47) + '...';
        }

        clickableElements.push({
            element: element,
            tag: element.tagName.toLowerCase(),
            text: innerText,
        });
    }

    for (let i = 0; i < element.children.length; i++) {
        walkDOM(element.children[i], clickableElements, processedElements);
    }
}
"#;

