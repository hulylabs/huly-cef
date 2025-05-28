/**
 * Platform-independent key codes for keyboard events
 */
export enum KeyCode {
  // Basic controls
  BACKSPACE = 8,
  TAB = 9,
  ENTER = 13,
  SHIFT = 16,
  CONTROL = 17,
  ALT = 18,
  PAUSE = 19,
  CAPS_LOCK = 20,
  ESCAPE = 27,
  SPACE = 32,
  PAGE_UP = 33,
  PAGE_DOWN = 34,
  END = 35,
  HOME = 36,

  // Arrow keys
  LEFT = 37,
  UP = 38,
  RIGHT = 39,
  DOWN = 40,

  // Special keys
  PRINT_SCREEN = 44,
  INSERT = 45,
  DELETE = 46,

  // Numbers
  KEY_0 = 48,
  KEY_1 = 49,
  KEY_2 = 50,
  KEY_3 = 51,
  KEY_4 = 52,
  KEY_5 = 53,
  KEY_6 = 54,
  KEY_7 = 55,
  KEY_8 = 56,
  KEY_9 = 57,

  // Letters
  KEY_A = 65,
  KEY_B = 66,
  KEY_C = 67,
  KEY_D = 68,
  KEY_E = 69,
  KEY_F = 70,
  KEY_G = 71,
  KEY_H = 72,
  KEY_I = 73,
  KEY_J = 74,
  KEY_K = 75,
  KEY_L = 76,
  KEY_M = 77,
  KEY_N = 78,
  KEY_O = 79,
  KEY_P = 80,
  KEY_Q = 81,
  KEY_R = 82,
  KEY_S = 83,
  KEY_T = 84,
  KEY_U = 85,
  KEY_V = 86,
  KEY_W = 87,
  KEY_X = 88,
  KEY_Y = 89,
  KEY_Z = 90,

  // Windows keys
  LEFT_WINDOWS = 91,
  RIGHT_WINDOWS = 92,
  CONTEXT_MENU = 93,

  // Numpad
  NUMPAD_0 = 96,
  NUMPAD_1 = 97,
  NUMPAD_2 = 98,
  NUMPAD_3 = 99,
  NUMPAD_4 = 100,
  NUMPAD_5 = 101,
  NUMPAD_6 = 102,
  NUMPAD_7 = 103,
  NUMPAD_8 = 104,
  NUMPAD_9 = 105,
  NUMPAD_MULTIPLY = 106,
  NUMPAD_ADD = 107,
  NUMPAD_SEPARATOR = 108,
  NUMPAD_SUBTRACT = 109,
  NUMPAD_DECIMAL = 110,
  NUMPAD_DIVIDE = 111,

  // Function keys
  F1 = 112,
  F2 = 113,
  F3 = 114,
  F4 = 115,
  F5 = 116,
  F6 = 117,
  F7 = 118,
  F8 = 119,
  F9 = 120,
  F10 = 121,
  F11 = 122,
  F12 = 123,
  F13 = 124,
  F14 = 125,
  F15 = 126,
  F16 = 127,
  F17 = 128,
  F18 = 129,
  F19 = 130,
  F20 = 131,
  F21 = 132,
  F22 = 133,
  F23 = 134,
  F24 = 135,

  // Lock keys
  NUM_LOCK = 144,
  SCROLL_LOCK = 145,

  // Shift keys
  LEFT_SHIFT = 160,
  RIGHT_SHIFT = 161,
  LEFT_CONTROL = 162,
  RIGHT_CONTROL = 163,
  LEFT_ALT = 164,
  RIGHT_ALT = 165,

  // Browser keys
  BROWSER_BACK = 166,
  BROWSER_FORWARD = 167,
  BROWSER_REFRESH = 168,
  BROWSER_STOP = 169,
  BROWSER_SEARCH = 170,
  BROWSER_FAVORITES = 171,
  BROWSER_HOME = 172,

  // Volume keys
  VOLUME_MUTE = 173,
  VOLUME_DOWN = 174,
  VOLUME_UP = 175,

  // Media keys
  MEDIA_NEXT_TRACK = 176,
  MEDIA_PREV_TRACK = 177,
  MEDIA_STOP = 178,
  MEDIA_PLAY_PAUSE = 179,
  MEDIA_LAUNCH_MAIL = 180,
  MEDIA_LAUNCH_MEDIA_SELECT = 181,
  MEDIA_LAUNCH_APP1 = 182,
  MEDIA_LAUNCH_APP2 = 183,

  // Punctuation and special characters
  SEMICOLON = 186, // ;:
  EQUAL = 187, // =+
  COMMA = 188, // ,<
  MINUS = 189, // -_
  PERIOD = 190, // .>
  SLASH = 191, // /?
  BACKQUOTE = 192, // `~
  BRACKET_LEFT = 219, // [{
  BACKSLASH = 220, // \|
  BRACKET_RIGHT = 221, // ]}
  QUOTE = 222, // '"

  // Additional keys
  META = 224,
  ALTGR = 225,

  // For any keys not covered above
  UNKNOWN = 0,
}

/**
 * Windows Virtual Key Codes
 * Reference: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
 */
export enum WindowsVirtualKeyCode {
  // Mouse buttons
  VK_LBUTTON = 0x01, // Left mouse button
  VK_RBUTTON = 0x02, // Right mouse button
  VK_CANCEL = 0x03, // Control-break processing
  VK_MBUTTON = 0x04, // Middle mouse button
  VK_XBUTTON1 = 0x05, // X1 mouse button
  VK_XBUTTON2 = 0x06, // X2 mouse button

  // Control keys
  VK_BACK = 0x08, // BACKSPACE key
  VK_TAB = 0x09, // TAB key
  VK_CLEAR = 0x0c, // CLEAR key
  VK_RETURN = 0x0d, // ENTER key
  VK_SHIFT = 0x10, // SHIFT key
  VK_CONTROL = 0x11, // CTRL key
  VK_MENU = 0x12, // ALT key
  VK_PAUSE = 0x13, // PAUSE key
  VK_CAPITAL = 0x14, // CAPS LOCK key

  // IME keys
  VK_KANA = 0x15, // IME Kana mode
  VK_HANGUL = 0x15, // IME Hangul mode
  VK_JUNJA = 0x17, // IME Junja mode
  VK_FINAL = 0x18, // IME final mode
  VK_HANJA = 0x19, // IME Hanja mode
  VK_KANJI = 0x19, // IME Kanji mode

  // Navigation and command keys
  VK_ESCAPE = 0x1b, // ESC key
  VK_CONVERT = 0x1c, // IME convert
  VK_NONCONVERT = 0x1d, // IME nonconvert
  VK_ACCEPT = 0x1e, // IME accept
  VK_MODECHANGE = 0x1f, // IME mode change request
  VK_SPACE = 0x20, // SPACEBAR
  VK_PRIOR = 0x21, // PAGE UP key
  VK_NEXT = 0x22, // PAGE DOWN key
  VK_END = 0x23, // END key
  VK_HOME = 0x24, // HOME key
  VK_LEFT = 0x25, // LEFT ARROW key
  VK_UP = 0x26, // UP ARROW key
  VK_RIGHT = 0x27, // RIGHT ARROW key
  VK_DOWN = 0x28, // DOWN ARROW key
  VK_SELECT = 0x29, // SELECT key
  VK_PRINT = 0x2a, // PRINT key
  VK_EXECUTE = 0x2b, // EXECUTE key
  VK_SNAPSHOT = 0x2c, // PRINT SCREEN key
  VK_INSERT = 0x2d, // INS key
  VK_DELETE = 0x2e, // DEL key
  VK_HELP = 0x2f, // HELP key

  // Number keys (0-9)
  VK_0 = 0x30, // 0 key
  VK_1 = 0x31, // 1 key
  VK_2 = 0x32, // 2 key
  VK_3 = 0x33, // 3 key
  VK_4 = 0x34, // 4 key
  VK_5 = 0x35, // 5 key
  VK_6 = 0x36, // 6 key
  VK_7 = 0x37, // 7 key
  VK_8 = 0x38, // 8 key
  VK_9 = 0x39, // 9 key

  // Letter keys (A-Z)
  VK_A = 0x41, // A key
  VK_B = 0x42, // B key
  VK_C = 0x43, // C key
  VK_D = 0x44, // D key
  VK_E = 0x45, // E key
  VK_F = 0x46, // F key
  VK_G = 0x47, // G key
  VK_H = 0x48, // H key
  VK_I = 0x49, // I key
  VK_J = 0x4a, // J key
  VK_K = 0x4b, // K key
  VK_L = 0x4c, // L key
  VK_M = 0x4d, // M key
  VK_N = 0x4e, // N key
  VK_O = 0x4f, // O key
  VK_P = 0x50, // P key
  VK_Q = 0x51, // Q key
  VK_R = 0x52, // R key
  VK_S = 0x53, // S key
  VK_T = 0x54, // T key
  VK_U = 0x55, // U key
  VK_V = 0x56, // V key
  VK_W = 0x57, // W key
  VK_X = 0x58, // X key
  VK_Y = 0x59, // Y key
  VK_Z = 0x5a, // Z key

  // Windows keys
  VK_LWIN = 0x5b, // Left Windows key
  VK_RWIN = 0x5c, // Right Windows key
  VK_APPS = 0x5d, // Applications key
  VK_SLEEP = 0x5f, // Computer Sleep key

  // Numeric keypad
  VK_NUMPAD0 = 0x60, // Numeric keypad 0 key
  VK_NUMPAD1 = 0x61, // Numeric keypad 1 key
  VK_NUMPAD2 = 0x62, // Numeric keypad 2 key
  VK_NUMPAD3 = 0x63, // Numeric keypad 3 key
  VK_NUMPAD4 = 0x64, // Numeric keypad 4 key
  VK_NUMPAD5 = 0x65, // Numeric keypad 5 key
  VK_NUMPAD6 = 0x66, // Numeric keypad 6 key
  VK_NUMPAD7 = 0x67, // Numeric keypad 7 key
  VK_NUMPAD8 = 0x68, // Numeric keypad 8 key
  VK_NUMPAD9 = 0x69, // Numeric keypad 9 key
  VK_MULTIPLY = 0x6a, // Multiply key
  VK_ADD = 0x6b, // Add key
  VK_SEPARATOR = 0x6c, // Separator key
  VK_SUBTRACT = 0x6d, // Subtract key
  VK_DECIMAL = 0x6e, // Decimal key
  VK_DIVIDE = 0x6f, // Divide key

  // Function keys
  VK_F1 = 0x70, // F1 key
  VK_F2 = 0x71, // F2 key
  VK_F3 = 0x72, // F3 key
  VK_F4 = 0x73, // F4 key
  VK_F5 = 0x74, // F5 key
  VK_F6 = 0x75, // F6 key
  VK_F7 = 0x76, // F7 key
  VK_F8 = 0x77, // F8 key
  VK_F9 = 0x78, // F9 key
  VK_F10 = 0x79, // F10 key
  VK_F11 = 0x7a, // F11 key
  VK_F12 = 0x7b, // F12 key
  VK_F13 = 0x7c, // F13 key
  VK_F14 = 0x7d, // F14 key
  VK_F15 = 0x7e, // F15 key
  VK_F16 = 0x7f, // F16 key
  VK_F17 = 0x80, // F17 key
  VK_F18 = 0x81, // F18 key
  VK_F19 = 0x82, // F19 key
  VK_F20 = 0x83, // F20 key
  VK_F21 = 0x84, // F21 key
  VK_F22 = 0x85, // F22 key
  VK_F23 = 0x86, // F23 key
  VK_F24 = 0x87, // F24 key

  // Lock keys
  VK_NUMLOCK = 0x90, // NUM LOCK key
  VK_SCROLL = 0x91, // SCROLL LOCK key

  // Shift keys
  VK_LSHIFT = 0xa0, // Left SHIFT key
  VK_RSHIFT = 0xa1, // Right SHIFT key
  VK_LCONTROL = 0xa2, // Left CONTROL key
  VK_RCONTROL = 0xa3, // Right CONTROL key
  VK_LMENU = 0xa4, // Left ALT key
  VK_RMENU = 0xa5, // Right ALT key

  // Browser keys
  VK_BROWSER_BACK = 0xa6, // Browser Back key
  VK_BROWSER_FORWARD = 0xa7, // Browser Forward key
  VK_BROWSER_REFRESH = 0xa8, // Browser Refresh key
  VK_BROWSER_STOP = 0xa9, // Browser Stop key
  VK_BROWSER_SEARCH = 0xaa, // Browser Search key
  VK_BROWSER_FAVORITES = 0xab, // Browser Favorites key
  VK_BROWSER_HOME = 0xac, // Browser Start and Home key

  // Volume keys
  VK_VOLUME_MUTE = 0xad, // Volume Mute key
  VK_VOLUME_DOWN = 0xae, // Volume Down key
  VK_VOLUME_UP = 0xaf, // Volume Up key

  // Media keys
  VK_MEDIA_NEXT_TRACK = 0xb0, // Next Track key
  VK_MEDIA_PREV_TRACK = 0xb1, // Previous Track key
  VK_MEDIA_STOP = 0xb2, // Stop Media key
  VK_MEDIA_PLAY_PAUSE = 0xb3, // Play/Pause Media key
  VK_LAUNCH_MAIL = 0xb4, // Start Mail key
  VK_LAUNCH_MEDIA_SELECT = 0xb5, // Select Media key
  VK_LAUNCH_APP1 = 0xb6, // Start Application 1 key
  VK_LAUNCH_APP2 = 0xb7, // Start Application 2 key

  // OEM keys
  VK_OEM_1 = 0xba, // For US: ;: key
  VK_OEM_PLUS = 0xbb, // For any country/region: +
  VK_OEM_COMMA = 0xbc, // For any country/region: , key
  VK_OEM_MINUS = 0xbd, // For any country/region: - key
  VK_OEM_PERIOD = 0xbe, // For any country/region: . key
  VK_OEM_2 = 0xbf, // For US: /? key
  VK_OEM_3 = 0xc0, // For US: `~ key
  VK_OEM_4 = 0xdb, // For US: [{ key
  VK_OEM_5 = 0xdc, // For US: \| key
  VK_OEM_6 = 0xdd, // For US: ]} key
  VK_OEM_7 = 0xde, // For US: '" key
  VK_OEM_8 = 0xdf, // Miscellaneous characters
  VK_OEM_102 = 0xe2, // Angle bracket/backslash key on RT 102-key keyboard

  // Processing and packet keys
  VK_PROCESSKEY = 0xe5, // IME PROCESS key
  VK_PACKET = 0xe7, // Used for Unicode characters

  // Special keys
  VK_ATTN = 0xf6, // Attn key
  VK_CRSEL = 0xf7, // CrSel key
  VK_EXSEL = 0xf8, // ExSel key
  VK_EREOF = 0xf9, // Erase EOF key
  VK_PLAY = 0xfa, // Play key
  VK_ZOOM = 0xfb, // Zoom key
  VK_NONAME = 0xfc, // Reserved
  VK_PA1 = 0xfd, // PA1 key
  VK_OEM_CLEAR = 0xfe, // Clear key

  // For any keys not covered above
  VK_UNKNOWN = 0,
}

/**
 * Converts a platform-independent KeyCode to a Windows virtual key code
 * @param keyCode The platform-independent key code to convert
 * @returns The corresponding Windows virtual key code
 */
export function keyCodeToWindowsVirtualKey(
  keyCode: KeyCode,
): WindowsVirtualKeyCode {
  switch (keyCode) {
    case KeyCode.BACKSPACE:
      return WindowsVirtualKeyCode.VK_BACK;
    case KeyCode.TAB:
      return WindowsVirtualKeyCode.VK_TAB;
    case KeyCode.ENTER:
      return WindowsVirtualKeyCode.VK_RETURN;
    case KeyCode.SHIFT:
      return WindowsVirtualKeyCode.VK_SHIFT;
    case KeyCode.CONTROL:
      return WindowsVirtualKeyCode.VK_CONTROL;
    case KeyCode.ALT:
      return WindowsVirtualKeyCode.VK_MENU;
    case KeyCode.PAUSE:
      return WindowsVirtualKeyCode.VK_PAUSE;
    case KeyCode.CAPS_LOCK:
      return WindowsVirtualKeyCode.VK_CAPITAL;
    case KeyCode.ESCAPE:
      return WindowsVirtualKeyCode.VK_ESCAPE;
    case KeyCode.SPACE:
      return WindowsVirtualKeyCode.VK_SPACE;
    case KeyCode.PAGE_UP:
      return WindowsVirtualKeyCode.VK_PRIOR;
    case KeyCode.PAGE_DOWN:
      return WindowsVirtualKeyCode.VK_NEXT;
    case KeyCode.END:
      return WindowsVirtualKeyCode.VK_END;
    case KeyCode.HOME:
      return WindowsVirtualKeyCode.VK_HOME;
    case KeyCode.LEFT:
      return WindowsVirtualKeyCode.VK_LEFT;
    case KeyCode.UP:
      return WindowsVirtualKeyCode.VK_UP;
    case KeyCode.RIGHT:
      return WindowsVirtualKeyCode.VK_RIGHT;
    case KeyCode.DOWN:
      return WindowsVirtualKeyCode.VK_DOWN;
    case KeyCode.PRINT_SCREEN:
      return WindowsVirtualKeyCode.VK_SNAPSHOT;
    case KeyCode.INSERT:
      return WindowsVirtualKeyCode.VK_INSERT;
    case KeyCode.DELETE:
      return WindowsVirtualKeyCode.VK_DELETE;
    case KeyCode.KEY_0:
      return WindowsVirtualKeyCode.VK_0;
    case KeyCode.KEY_1:
      return WindowsVirtualKeyCode.VK_1;
    case KeyCode.KEY_2:
      return WindowsVirtualKeyCode.VK_2;
    case KeyCode.KEY_3:
      return WindowsVirtualKeyCode.VK_3;
    case KeyCode.KEY_4:
      return WindowsVirtualKeyCode.VK_4;
    case KeyCode.KEY_5:
      return WindowsVirtualKeyCode.VK_5;
    case KeyCode.KEY_6:
      return WindowsVirtualKeyCode.VK_6;
    case KeyCode.KEY_7:
      return WindowsVirtualKeyCode.VK_7;
    case KeyCode.KEY_8:
      return WindowsVirtualKeyCode.VK_8;
    case KeyCode.KEY_9:
      return WindowsVirtualKeyCode.VK_9;
    case KeyCode.KEY_A:
      return WindowsVirtualKeyCode.VK_A;
    case KeyCode.KEY_B:
      return WindowsVirtualKeyCode.VK_B;
    case KeyCode.KEY_C:
      return WindowsVirtualKeyCode.VK_C;
    case KeyCode.KEY_D:
      return WindowsVirtualKeyCode.VK_D;
    case KeyCode.KEY_E:
      return WindowsVirtualKeyCode.VK_E;
    case KeyCode.KEY_F:
      return WindowsVirtualKeyCode.VK_F;
    case KeyCode.KEY_G:
      return WindowsVirtualKeyCode.VK_G;
    case KeyCode.KEY_H:
      return WindowsVirtualKeyCode.VK_H;
    case KeyCode.KEY_I:
      return WindowsVirtualKeyCode.VK_I;
    case KeyCode.KEY_J:
      return WindowsVirtualKeyCode.VK_J;
    case KeyCode.KEY_K:
      return WindowsVirtualKeyCode.VK_K;
    case KeyCode.KEY_L:
      return WindowsVirtualKeyCode.VK_L;
    case KeyCode.KEY_M:
      return WindowsVirtualKeyCode.VK_M;
    case KeyCode.KEY_N:
      return WindowsVirtualKeyCode.VK_N;
    case KeyCode.KEY_O:
      return WindowsVirtualKeyCode.VK_O;
    case KeyCode.KEY_P:
      return WindowsVirtualKeyCode.VK_P;
    case KeyCode.KEY_Q:
      return WindowsVirtualKeyCode.VK_Q;
    case KeyCode.KEY_R:
      return WindowsVirtualKeyCode.VK_R;
    case KeyCode.KEY_S:
      return WindowsVirtualKeyCode.VK_S;
    case KeyCode.KEY_T:
      return WindowsVirtualKeyCode.VK_T;
    case KeyCode.KEY_U:
      return WindowsVirtualKeyCode.VK_U;
    case KeyCode.KEY_V:
      return WindowsVirtualKeyCode.VK_V;
    case KeyCode.KEY_W:
      return WindowsVirtualKeyCode.VK_W;
    case KeyCode.KEY_X:
      return WindowsVirtualKeyCode.VK_X;
    case KeyCode.KEY_Y:
      return WindowsVirtualKeyCode.VK_Y;
    case KeyCode.KEY_Z:
      return WindowsVirtualKeyCode.VK_Z;
    case KeyCode.LEFT_WINDOWS:
      return WindowsVirtualKeyCode.VK_LWIN;
    case KeyCode.RIGHT_WINDOWS:
      return WindowsVirtualKeyCode.VK_RWIN;
    case KeyCode.CONTEXT_MENU:
      return WindowsVirtualKeyCode.VK_APPS;
    case KeyCode.NUMPAD_0:
      return WindowsVirtualKeyCode.VK_NUMPAD0;
    case KeyCode.NUMPAD_1:
      return WindowsVirtualKeyCode.VK_NUMPAD1;
    case KeyCode.NUMPAD_2:
      return WindowsVirtualKeyCode.VK_NUMPAD2;
    case KeyCode.NUMPAD_3:
      return WindowsVirtualKeyCode.VK_NUMPAD3;
    case KeyCode.NUMPAD_4:
      return WindowsVirtualKeyCode.VK_NUMPAD4;
    case KeyCode.NUMPAD_5:
      return WindowsVirtualKeyCode.VK_NUMPAD5;
    case KeyCode.NUMPAD_6:
      return WindowsVirtualKeyCode.VK_NUMPAD6;
    case KeyCode.NUMPAD_7:
      return WindowsVirtualKeyCode.VK_NUMPAD7;
    case KeyCode.NUMPAD_8:
      return WindowsVirtualKeyCode.VK_NUMPAD8;
    case KeyCode.NUMPAD_9:
      return WindowsVirtualKeyCode.VK_NUMPAD9;
    case KeyCode.NUMPAD_MULTIPLY:
      return WindowsVirtualKeyCode.VK_MULTIPLY;
    case KeyCode.NUMPAD_ADD:
      return WindowsVirtualKeyCode.VK_ADD;
    case KeyCode.NUMPAD_SEPARATOR:
      return WindowsVirtualKeyCode.VK_SEPARATOR;
    case KeyCode.NUMPAD_SUBTRACT:
      return WindowsVirtualKeyCode.VK_SUBTRACT;
    case KeyCode.NUMPAD_DECIMAL:
      return WindowsVirtualKeyCode.VK_DECIMAL;
    case KeyCode.NUMPAD_DIVIDE:
      return WindowsVirtualKeyCode.VK_DIVIDE;
    case KeyCode.F1:
      return WindowsVirtualKeyCode.VK_F1;
    case KeyCode.F2:
      return WindowsVirtualKeyCode.VK_F2;
    case KeyCode.F3:
      return WindowsVirtualKeyCode.VK_F3;
    case KeyCode.F4:
      return WindowsVirtualKeyCode.VK_F4;
    case KeyCode.F5:
      return WindowsVirtualKeyCode.VK_F5;
    case KeyCode.F6:
      return WindowsVirtualKeyCode.VK_F6;
    case KeyCode.F7:
      return WindowsVirtualKeyCode.VK_F7;
    case KeyCode.F8:
      return WindowsVirtualKeyCode.VK_F8;
    case KeyCode.F9:
      return WindowsVirtualKeyCode.VK_F9;
    case KeyCode.F10:
      return WindowsVirtualKeyCode.VK_F10;
    case KeyCode.F11:
      return WindowsVirtualKeyCode.VK_F11;
    case KeyCode.F12:
      return WindowsVirtualKeyCode.VK_F12;
    case KeyCode.F13:
      return WindowsVirtualKeyCode.VK_F13;
    case KeyCode.F14:
      return WindowsVirtualKeyCode.VK_F14;
    case KeyCode.F15:
      return WindowsVirtualKeyCode.VK_F15;
    case KeyCode.F16:
      return WindowsVirtualKeyCode.VK_F16;
    case KeyCode.F17:
      return WindowsVirtualKeyCode.VK_F17;
    case KeyCode.F18:
      return WindowsVirtualKeyCode.VK_F18;
    case KeyCode.F19:
      return WindowsVirtualKeyCode.VK_F19;
    case KeyCode.F20:
      return WindowsVirtualKeyCode.VK_F20;
    case KeyCode.F21:
      return WindowsVirtualKeyCode.VK_F21;
    case KeyCode.F22:
      return WindowsVirtualKeyCode.VK_F22;
    case KeyCode.F23:
      return WindowsVirtualKeyCode.VK_F23;
    case KeyCode.F24:
      return WindowsVirtualKeyCode.VK_F24;
    case KeyCode.NUM_LOCK:
      return WindowsVirtualKeyCode.VK_NUMLOCK;
    case KeyCode.SCROLL_LOCK:
      return WindowsVirtualKeyCode.VK_SCROLL;
    case KeyCode.LEFT_SHIFT:
      return WindowsVirtualKeyCode.VK_LSHIFT;
    case KeyCode.RIGHT_SHIFT:
      return WindowsVirtualKeyCode.VK_RSHIFT;
    case KeyCode.LEFT_CONTROL:
      return WindowsVirtualKeyCode.VK_LCONTROL;
    case KeyCode.RIGHT_CONTROL:
      return WindowsVirtualKeyCode.VK_RCONTROL;
    case KeyCode.LEFT_ALT:
      return WindowsVirtualKeyCode.VK_LMENU;
    case KeyCode.RIGHT_ALT:
      return WindowsVirtualKeyCode.VK_RMENU;
    case KeyCode.BROWSER_BACK:
      return WindowsVirtualKeyCode.VK_BROWSER_BACK;
    case KeyCode.BROWSER_FORWARD:
      return WindowsVirtualKeyCode.VK_BROWSER_FORWARD;
    case KeyCode.BROWSER_REFRESH:
      return WindowsVirtualKeyCode.VK_BROWSER_REFRESH;
    case KeyCode.BROWSER_STOP:
      return WindowsVirtualKeyCode.VK_BROWSER_STOP;
    case KeyCode.BROWSER_SEARCH:
      return WindowsVirtualKeyCode.VK_BROWSER_SEARCH;
    case KeyCode.BROWSER_FAVORITES:
      return WindowsVirtualKeyCode.VK_BROWSER_FAVORITES;
    case KeyCode.BROWSER_HOME:
      return WindowsVirtualKeyCode.VK_BROWSER_HOME;
    case KeyCode.VOLUME_MUTE:
      return WindowsVirtualKeyCode.VK_VOLUME_MUTE;
    case KeyCode.VOLUME_DOWN:
      return WindowsVirtualKeyCode.VK_VOLUME_DOWN;
    case KeyCode.VOLUME_UP:
      return WindowsVirtualKeyCode.VK_VOLUME_UP;
    case KeyCode.MEDIA_NEXT_TRACK:
      return WindowsVirtualKeyCode.VK_MEDIA_NEXT_TRACK;
    case KeyCode.MEDIA_PREV_TRACK:
      return WindowsVirtualKeyCode.VK_MEDIA_PREV_TRACK;
    case KeyCode.MEDIA_STOP:
      return WindowsVirtualKeyCode.VK_MEDIA_STOP;
    case KeyCode.MEDIA_PLAY_PAUSE:
      return WindowsVirtualKeyCode.VK_MEDIA_PLAY_PAUSE;
    case KeyCode.MEDIA_LAUNCH_MAIL:
      return WindowsVirtualKeyCode.VK_LAUNCH_MAIL;
    case KeyCode.MEDIA_LAUNCH_MEDIA_SELECT:
      return WindowsVirtualKeyCode.VK_LAUNCH_MEDIA_SELECT;
    case KeyCode.MEDIA_LAUNCH_APP1:
      return WindowsVirtualKeyCode.VK_LAUNCH_APP1;
    case KeyCode.MEDIA_LAUNCH_APP2:
      return WindowsVirtualKeyCode.VK_LAUNCH_APP2;
    case KeyCode.SEMICOLON:
      return WindowsVirtualKeyCode.VK_OEM_1;
    case KeyCode.EQUAL:
      return WindowsVirtualKeyCode.VK_OEM_PLUS;
    case KeyCode.COMMA:
      return WindowsVirtualKeyCode.VK_OEM_COMMA;
    case KeyCode.MINUS:
      return WindowsVirtualKeyCode.VK_OEM_MINUS;
    case KeyCode.PERIOD:
      return WindowsVirtualKeyCode.VK_OEM_PERIOD;
    case KeyCode.SLASH:
      return WindowsVirtualKeyCode.VK_OEM_2;
    case KeyCode.BACKQUOTE:
      return WindowsVirtualKeyCode.VK_OEM_3;
    case KeyCode.BRACKET_LEFT:
      return WindowsVirtualKeyCode.VK_OEM_4;
    case KeyCode.BACKSLASH:
      return WindowsVirtualKeyCode.VK_OEM_5;
    case KeyCode.BRACKET_RIGHT:
      return WindowsVirtualKeyCode.VK_OEM_6;
    case KeyCode.QUOTE:
      return WindowsVirtualKeyCode.VK_OEM_7;

    // Default to unknown for any keys not explicitly mapped
    default:
      return WindowsVirtualKeyCode.VK_UNKNOWN;
  }
}

/**
 * macOS Virtual Key Codes based on the Carbon framework
 * Reference: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h
 */
export enum MacOSVirtualKeyCode {
  // Keyboard
  kVK_ANSI_A = 0x00,
  kVK_ANSI_S = 0x01,
  kVK_ANSI_D = 0x02,
  kVK_ANSI_F = 0x03,
  kVK_ANSI_H = 0x04,
  kVK_ANSI_G = 0x05,
  kVK_ANSI_Z = 0x06,
  kVK_ANSI_X = 0x07,
  kVK_ANSI_C = 0x08,
  kVK_ANSI_V = 0x09,
  kVK_ANSI_B = 0x0b,
  kVK_ANSI_Q = 0x0c,
  kVK_ANSI_W = 0x0d,
  kVK_ANSI_E = 0x0e,
  kVK_ANSI_R = 0x0f,
  kVK_ANSI_Y = 0x10,
  kVK_ANSI_T = 0x11,
  kVK_ANSI_1 = 0x12,
  kVK_ANSI_2 = 0x13,
  kVK_ANSI_3 = 0x14,
  kVK_ANSI_4 = 0x15,
  kVK_ANSI_6 = 0x16,
  kVK_ANSI_5 = 0x17,
  kVK_ANSI_Equal = 0x18,
  kVK_ANSI_9 = 0x19,
  kVK_ANSI_7 = 0x1a,
  kVK_ANSI_Minus = 0x1b,
  kVK_ANSI_8 = 0x1c,
  kVK_ANSI_0 = 0x1d,
  kVK_ANSI_RightBracket = 0x1e,
  kVK_ANSI_O = 0x1f,
  kVK_ANSI_U = 0x20,
  kVK_ANSI_LeftBracket = 0x21,
  kVK_ANSI_I = 0x22,
  kVK_ANSI_P = 0x23,
  kVK_Return = 0x24,
  kVK_ANSI_L = 0x25,
  kVK_ANSI_J = 0x26,
  kVK_ANSI_Quote = 0x27,
  kVK_ANSI_K = 0x28,
  kVK_ANSI_Semicolon = 0x29,
  kVK_ANSI_Backslash = 0x2a,
  kVK_ANSI_Comma = 0x2b,
  kVK_ANSI_Slash = 0x2c,
  kVK_ANSI_N = 0x2d,
  kVK_ANSI_M = 0x2e,
  kVK_ANSI_Period = 0x2f,
  kVK_Tab = 0x30,
  kVK_Space = 0x31,
  kVK_ANSI_Grave = 0x32,
  kVK_Delete = 0x33,
  kVK_Escape = 0x35,
  kVK_Command = 0x37,
  kVK_Shift = 0x38,
  kVK_CapsLock = 0x39,
  kVK_Option = 0x3a,
  kVK_Control = 0x3b,
  kVK_RightShift = 0x3c,
  kVK_RightOption = 0x3d,
  kVK_RightControl = 0x3e,
  kVK_Function = 0x3f,
  kVK_F17 = 0x40,
  kVK_VolumeUp = 0x48,
  kVK_VolumeDown = 0x49,
  kVK_Mute = 0x4a,
  kVK_F18 = 0x4f,
  kVK_F19 = 0x50,
  kVK_F20 = 0x5a,
  kVK_F5 = 0x60,
  kVK_F6 = 0x61,
  kVK_F7 = 0x62,
  kVK_F3 = 0x63,
  kVK_F8 = 0x64,
  kVK_F9 = 0x65,
  kVK_F11 = 0x67,
  kVK_F13 = 0x69,
  kVK_F16 = 0x6a,
  kVK_F14 = 0x6b,
  kVK_F10 = 0x6d,
  kVK_F12 = 0x6f,
  kVK_F15 = 0x71,
  kVK_Help = 0x72,
  kVK_Home = 0x73,
  kVK_PageUp = 0x74,
  kVK_ForwardDelete = 0x75,
  kVK_F4 = 0x76,
  kVK_End = 0x77,
  kVK_F2 = 0x78,
  kVK_PageDown = 0x79,
  kVK_F1 = 0x7a,
  kVK_LeftArrow = 0x7b,
  kVK_RightArrow = 0x7c,
  kVK_DownArrow = 0x7d,
  kVK_UpArrow = 0x7e,

  // Keypad
  kVK_ANSI_KeypadDecimal = 0x41,
  kVK_ANSI_KeypadMultiply = 0x43,
  kVK_ANSI_KeypadPlus = 0x45,
  kVK_ANSI_KeypadClear = 0x47,
  kVK_ANSI_KeypadDivide = 0x4b,
  kVK_ANSI_KeypadEnter = 0x4c,
  kVK_ANSI_KeypadMinus = 0x4e,
  kVK_ANSI_KeypadEquals = 0x51,
  kVK_ANSI_Keypad0 = 0x52,
  kVK_ANSI_Keypad1 = 0x53,
  kVK_ANSI_Keypad2 = 0x54,
  kVK_ANSI_Keypad3 = 0x55,
  kVK_ANSI_Keypad4 = 0x56,
  kVK_ANSI_Keypad5 = 0x57,
  kVK_ANSI_Keypad6 = 0x58,
  kVK_ANSI_Keypad7 = 0x59,
  kVK_ANSI_Keypad8 = 0x5b,
  kVK_ANSI_Keypad9 = 0x5c,

  // For any keys not covered above
  kVK_UNKNOWN = 0xff,
}

/**
 * Converts a platform-independent KeyCode to a macOS virtual key code
 * @param keyCode The platform-independent key code to convert
 * @returns The corresponding macOS virtual key code
 */
export function keyCodeToMacOSVirtualKey(
  keyCode: KeyCode,
): MacOSVirtualKeyCode {
  switch (keyCode) {
    case KeyCode.BACKSPACE:
      return MacOSVirtualKeyCode.kVK_Delete;
    case KeyCode.TAB:
      return MacOSVirtualKeyCode.kVK_Tab;
    case KeyCode.ENTER:
      return MacOSVirtualKeyCode.kVK_Return;
    case KeyCode.SHIFT:
      return MacOSVirtualKeyCode.kVK_Shift;
    case KeyCode.CONTROL:
      return MacOSVirtualKeyCode.kVK_Control;
    case KeyCode.ALT:
      return MacOSVirtualKeyCode.kVK_Option;
    case KeyCode.PAUSE:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.CAPS_LOCK:
      return MacOSVirtualKeyCode.kVK_CapsLock;
    case KeyCode.ESCAPE:
      return MacOSVirtualKeyCode.kVK_Escape;
    case KeyCode.SPACE:
      return MacOSVirtualKeyCode.kVK_Space;
    case KeyCode.PAGE_UP:
      return MacOSVirtualKeyCode.kVK_PageUp;
    case KeyCode.PAGE_DOWN:
      return MacOSVirtualKeyCode.kVK_PageDown;
    case KeyCode.END:
      return MacOSVirtualKeyCode.kVK_End;
    case KeyCode.HOME:
      return MacOSVirtualKeyCode.kVK_Home;
    case KeyCode.LEFT:
      return MacOSVirtualKeyCode.kVK_LeftArrow;
    case KeyCode.UP:
      return MacOSVirtualKeyCode.kVK_UpArrow;
    case KeyCode.RIGHT:
      return MacOSVirtualKeyCode.kVK_RightArrow;
    case KeyCode.DOWN:
      return MacOSVirtualKeyCode.kVK_DownArrow;
    case KeyCode.PRINT_SCREEN:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.INSERT:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.DELETE:
      return MacOSVirtualKeyCode.kVK_ForwardDelete;
    case KeyCode.KEY_0:
      return MacOSVirtualKeyCode.kVK_ANSI_0;
    case KeyCode.KEY_1:
      return MacOSVirtualKeyCode.kVK_ANSI_1;
    case KeyCode.KEY_2:
      return MacOSVirtualKeyCode.kVK_ANSI_2;
    case KeyCode.KEY_3:
      return MacOSVirtualKeyCode.kVK_ANSI_3;
    case KeyCode.KEY_4:
      return MacOSVirtualKeyCode.kVK_ANSI_4;
    case KeyCode.KEY_5:
      return MacOSVirtualKeyCode.kVK_ANSI_5;
    case KeyCode.KEY_6:
      return MacOSVirtualKeyCode.kVK_ANSI_6;
    case KeyCode.KEY_7:
      return MacOSVirtualKeyCode.kVK_ANSI_7;
    case KeyCode.KEY_8:
      return MacOSVirtualKeyCode.kVK_ANSI_8;
    case KeyCode.KEY_9:
      return MacOSVirtualKeyCode.kVK_ANSI_9;
    case KeyCode.KEY_A:
      return MacOSVirtualKeyCode.kVK_ANSI_A;
    case KeyCode.KEY_B:
      return MacOSVirtualKeyCode.kVK_ANSI_B;
    case KeyCode.KEY_C:
      return MacOSVirtualKeyCode.kVK_ANSI_C;
    case KeyCode.KEY_D:
      return MacOSVirtualKeyCode.kVK_ANSI_D;
    case KeyCode.KEY_E:
      return MacOSVirtualKeyCode.kVK_ANSI_E;
    case KeyCode.KEY_F:
      return MacOSVirtualKeyCode.kVK_ANSI_F;
    case KeyCode.KEY_G:
      return MacOSVirtualKeyCode.kVK_ANSI_G;
    case KeyCode.KEY_H:
      return MacOSVirtualKeyCode.kVK_ANSI_H;
    case KeyCode.KEY_I:
      return MacOSVirtualKeyCode.kVK_ANSI_I;
    case KeyCode.KEY_J:
      return MacOSVirtualKeyCode.kVK_ANSI_J;
    case KeyCode.KEY_K:
      return MacOSVirtualKeyCode.kVK_ANSI_K;
    case KeyCode.KEY_L:
      return MacOSVirtualKeyCode.kVK_ANSI_L;
    case KeyCode.KEY_M:
      return MacOSVirtualKeyCode.kVK_ANSI_M;
    case KeyCode.KEY_N:
      return MacOSVirtualKeyCode.kVK_ANSI_N;
    case KeyCode.KEY_O:
      return MacOSVirtualKeyCode.kVK_ANSI_O;
    case KeyCode.KEY_P:
      return MacOSVirtualKeyCode.kVK_ANSI_P;
    case KeyCode.KEY_Q:
      return MacOSVirtualKeyCode.kVK_ANSI_Q;
    case KeyCode.KEY_R:
      return MacOSVirtualKeyCode.kVK_ANSI_R;
    case KeyCode.KEY_S:
      return MacOSVirtualKeyCode.kVK_ANSI_S;
    case KeyCode.KEY_T:
      return MacOSVirtualKeyCode.kVK_ANSI_T;
    case KeyCode.KEY_U:
      return MacOSVirtualKeyCode.kVK_ANSI_U;
    case KeyCode.KEY_V:
      return MacOSVirtualKeyCode.kVK_ANSI_V;
    case KeyCode.KEY_W:
      return MacOSVirtualKeyCode.kVK_ANSI_W;
    case KeyCode.KEY_X:
      return MacOSVirtualKeyCode.kVK_ANSI_X;
    case KeyCode.KEY_Y:
      return MacOSVirtualKeyCode.kVK_ANSI_Y;
    case KeyCode.KEY_Z:
      return MacOSVirtualKeyCode.kVK_ANSI_Z;
    case KeyCode.LEFT_WINDOWS:
      return MacOSVirtualKeyCode.kVK_Command;
    case KeyCode.RIGHT_WINDOWS:
      return MacOSVirtualKeyCode.kVK_Command;
    case KeyCode.CONTEXT_MENU:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.NUMPAD_0:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad0;
    case KeyCode.NUMPAD_1:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad1;
    case KeyCode.NUMPAD_2:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad2;
    case KeyCode.NUMPAD_3:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad3;
    case KeyCode.NUMPAD_4:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad4;
    case KeyCode.NUMPAD_5:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad5;
    case KeyCode.NUMPAD_6:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad6;
    case KeyCode.NUMPAD_7:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad7;
    case KeyCode.NUMPAD_8:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad8;
    case KeyCode.NUMPAD_9:
      return MacOSVirtualKeyCode.kVK_ANSI_Keypad9;
    case KeyCode.NUMPAD_MULTIPLY:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadMultiply;
    case KeyCode.NUMPAD_ADD:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadPlus;
    case KeyCode.NUMPAD_SEPARATOR:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.NUMPAD_SUBTRACT:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadMinus;
    case KeyCode.NUMPAD_DECIMAL:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadDecimal;
    case KeyCode.NUMPAD_DIVIDE:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadDivide;
    case KeyCode.F1:
      return MacOSVirtualKeyCode.kVK_F1;
    case KeyCode.F2:
      return MacOSVirtualKeyCode.kVK_F2;
    case KeyCode.F3:
      return MacOSVirtualKeyCode.kVK_F3;
    case KeyCode.F4:
      return MacOSVirtualKeyCode.kVK_F4;
    case KeyCode.F5:
      return MacOSVirtualKeyCode.kVK_F5;
    case KeyCode.F6:
      return MacOSVirtualKeyCode.kVK_F6;
    case KeyCode.F7:
      return MacOSVirtualKeyCode.kVK_F7;
    case KeyCode.F8:
      return MacOSVirtualKeyCode.kVK_F8;
    case KeyCode.F9:
      return MacOSVirtualKeyCode.kVK_F9;
    case KeyCode.F10:
      return MacOSVirtualKeyCode.kVK_F10;
    case KeyCode.F11:
      return MacOSVirtualKeyCode.kVK_F11;
    case KeyCode.F12:
      return MacOSVirtualKeyCode.kVK_F12;
    case KeyCode.F13:
      return MacOSVirtualKeyCode.kVK_F13;
    case KeyCode.F14:
      return MacOSVirtualKeyCode.kVK_F14;
    case KeyCode.F15:
      return MacOSVirtualKeyCode.kVK_F15;
    case KeyCode.F16:
      return MacOSVirtualKeyCode.kVK_F16;
    case KeyCode.F17:
      return MacOSVirtualKeyCode.kVK_F17;
    case KeyCode.F18:
      return MacOSVirtualKeyCode.kVK_F18;
    case KeyCode.F19:
      return MacOSVirtualKeyCode.kVK_F19;
    case KeyCode.F20:
      return MacOSVirtualKeyCode.kVK_F20;
    case KeyCode.F21:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.F22:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.F23:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.F24:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.NUM_LOCK:
      return MacOSVirtualKeyCode.kVK_ANSI_KeypadClear;
    case KeyCode.SCROLL_LOCK:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.LEFT_SHIFT:
      return MacOSVirtualKeyCode.kVK_Shift;
    case KeyCode.RIGHT_SHIFT:
      return MacOSVirtualKeyCode.kVK_RightShift;
    case KeyCode.LEFT_CONTROL:
      return MacOSVirtualKeyCode.kVK_Control;
    case KeyCode.RIGHT_CONTROL:
      return MacOSVirtualKeyCode.kVK_RightControl;
    case KeyCode.LEFT_ALT:
      return MacOSVirtualKeyCode.kVK_Option;
    case KeyCode.RIGHT_ALT:
      return MacOSVirtualKeyCode.kVK_RightOption;
    case KeyCode.BROWSER_BACK:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_FORWARD:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_REFRESH:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_STOP:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_SEARCH:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_FAVORITES:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.BROWSER_HOME:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.VOLUME_MUTE:
      return MacOSVirtualKeyCode.kVK_Mute;
    case KeyCode.VOLUME_DOWN:
      return MacOSVirtualKeyCode.kVK_VolumeDown;
    case KeyCode.VOLUME_UP:
      return MacOSVirtualKeyCode.kVK_VolumeUp;
    case KeyCode.MEDIA_NEXT_TRACK:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_PREV_TRACK:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_STOP:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_PLAY_PAUSE:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_LAUNCH_MAIL:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_LAUNCH_MEDIA_SELECT:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_LAUNCH_APP1:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.MEDIA_LAUNCH_APP2:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
    case KeyCode.SEMICOLON:
      return MacOSVirtualKeyCode.kVK_ANSI_Semicolon;
    case KeyCode.EQUAL:
      return MacOSVirtualKeyCode.kVK_ANSI_Equal;
    case KeyCode.COMMA:
      return MacOSVirtualKeyCode.kVK_ANSI_Comma;
    case KeyCode.MINUS:
      return MacOSVirtualKeyCode.kVK_ANSI_Minus;
    case KeyCode.PERIOD:
      return MacOSVirtualKeyCode.kVK_ANSI_Period;
    case KeyCode.SLASH:
      return MacOSVirtualKeyCode.kVK_ANSI_Slash;
    case KeyCode.BACKQUOTE:
      return MacOSVirtualKeyCode.kVK_ANSI_Grave;
    case KeyCode.BRACKET_LEFT:
      return MacOSVirtualKeyCode.kVK_ANSI_LeftBracket;
    case KeyCode.BACKSLASH:
      return MacOSVirtualKeyCode.kVK_ANSI_Backslash;
    case KeyCode.BRACKET_RIGHT:
      return MacOSVirtualKeyCode.kVK_ANSI_RightBracket;
    case KeyCode.QUOTE:
      return MacOSVirtualKeyCode.kVK_ANSI_Quote;

    // For keys that don't have direct macOS equivalents
    default:
      return MacOSVirtualKeyCode.kVK_UNKNOWN;
  }
}
