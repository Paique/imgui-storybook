//! Browser `KeyboardEvent.code` → Dear ImGui key. Rust mirror of `host/net/InputInjector.java`
//! (same coverage: letters, digits, arrows, modifiers and control keys).

use easy_imgui_sys as sys;

/// Returns the `ImGuiKey` constant for a browser key code, or `None` when unmapped
/// (unmapped keys are ignored, like the Java host).
pub fn map_key(code: &str) -> Option<sys::ImGuiKey> {
    use sys::ImGuiKey;
    let key = match code {
        // Letters
        "KeyA" => ImGuiKey::ImGuiKey_A,
        "KeyB" => ImGuiKey::ImGuiKey_B,
        "KeyC" => ImGuiKey::ImGuiKey_C,
        "KeyD" => ImGuiKey::ImGuiKey_D,
        "KeyE" => ImGuiKey::ImGuiKey_E,
        "KeyF" => ImGuiKey::ImGuiKey_F,
        "KeyG" => ImGuiKey::ImGuiKey_G,
        "KeyH" => ImGuiKey::ImGuiKey_H,
        "KeyI" => ImGuiKey::ImGuiKey_I,
        "KeyJ" => ImGuiKey::ImGuiKey_J,
        "KeyK" => ImGuiKey::ImGuiKey_K,
        "KeyL" => ImGuiKey::ImGuiKey_L,
        "KeyM" => ImGuiKey::ImGuiKey_M,
        "KeyN" => ImGuiKey::ImGuiKey_N,
        "KeyO" => ImGuiKey::ImGuiKey_O,
        "KeyP" => ImGuiKey::ImGuiKey_P,
        "KeyQ" => ImGuiKey::ImGuiKey_Q,
        "KeyR" => ImGuiKey::ImGuiKey_R,
        "KeyS" => ImGuiKey::ImGuiKey_S,
        "KeyT" => ImGuiKey::ImGuiKey_T,
        "KeyU" => ImGuiKey::ImGuiKey_U,
        "KeyV" => ImGuiKey::ImGuiKey_V,
        "KeyW" => ImGuiKey::ImGuiKey_W,
        "KeyX" => ImGuiKey::ImGuiKey_X,
        "KeyY" => ImGuiKey::ImGuiKey_Y,
        "KeyZ" => ImGuiKey::ImGuiKey_Z,
        // Digits (imgui-java named these `_0.._9`; upstream C++ is `0..9`)
        "Digit0" => ImGuiKey::ImGuiKey_0,
        "Digit1" => ImGuiKey::ImGuiKey_1,
        "Digit2" => ImGuiKey::ImGuiKey_2,
        "Digit3" => ImGuiKey::ImGuiKey_3,
        "Digit4" => ImGuiKey::ImGuiKey_4,
        "Digit5" => ImGuiKey::ImGuiKey_5,
        "Digit6" => ImGuiKey::ImGuiKey_6,
        "Digit7" => ImGuiKey::ImGuiKey_7,
        "Digit8" => ImGuiKey::ImGuiKey_8,
        "Digit9" => ImGuiKey::ImGuiKey_9,
        // Arrows
        "ArrowLeft" => ImGuiKey::ImGuiKey_LeftArrow,
        "ArrowRight" => ImGuiKey::ImGuiKey_RightArrow,
        "ArrowUp" => ImGuiKey::ImGuiKey_UpArrow,
        "ArrowDown" => ImGuiKey::ImGuiKey_DownArrow,
        // Modifiers
        "ShiftLeft" => ImGuiKey::ImGuiKey_LeftShift,
        "ShiftRight" => ImGuiKey::ImGuiKey_RightShift,
        "ControlLeft" => ImGuiKey::ImGuiKey_LeftCtrl,
        "ControlRight" => ImGuiKey::ImGuiKey_RightCtrl,
        "AltLeft" => ImGuiKey::ImGuiKey_LeftAlt,
        "AltRight" => ImGuiKey::ImGuiKey_RightAlt,
        "MetaLeft" => ImGuiKey::ImGuiKey_LeftSuper,
        "MetaRight" => ImGuiKey::ImGuiKey_RightSuper,
        // Control keys pass through with the same name
        "Escape" => ImGuiKey::ImGuiKey_Escape,
        "Enter" => ImGuiKey::ImGuiKey_Enter,
        "Space" => ImGuiKey::ImGuiKey_Space,
        "Tab" => ImGuiKey::ImGuiKey_Tab,
        "Backspace" => ImGuiKey::ImGuiKey_Backspace,
        "Delete" => ImGuiKey::ImGuiKey_Delete,
        "Home" => ImGuiKey::ImGuiKey_Home,
        "End" => ImGuiKey::ImGuiKey_End,
        "PageUp" => ImGuiKey::ImGuiKey_PageUp,
        "PageDown" => ImGuiKey::ImGuiKey_PageDown,
        "Insert" => ImGuiKey::ImGuiKey_Insert,
        "F1" => ImGuiKey::ImGuiKey_F1,
        "F2" => ImGuiKey::ImGuiKey_F2,
        "F3" => ImGuiKey::ImGuiKey_F3,
        "F4" => ImGuiKey::ImGuiKey_F4,
        "F5" => ImGuiKey::ImGuiKey_F5,
        "F6" => ImGuiKey::ImGuiKey_F6,
        "F7" => ImGuiKey::ImGuiKey_F7,
        "F8" => ImGuiKey::ImGuiKey_F8,
        "F9" => ImGuiKey::ImGuiKey_F9,
        "F10" => ImGuiKey::ImGuiKey_F10,
        "F11" => ImGuiKey::ImGuiKey_F11,
        "F12" => ImGuiKey::ImGuiKey_F12,
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_codes_map() {
        assert!(map_key("KeyA").is_some());
        assert_eq!(map_key("Digit0"), Some(sys::ImGuiKey::ImGuiKey_0));
        assert_eq!(map_key("ArrowLeft"), Some(sys::ImGuiKey::ImGuiKey_LeftArrow));
        assert_eq!(map_key("ShiftLeft"), Some(sys::ImGuiKey::ImGuiKey_LeftShift));
        assert_eq!(map_key("MetaLeft"), Some(sys::ImGuiKey::ImGuiKey_LeftSuper));
        assert_eq!(map_key("F12"), Some(sys::ImGuiKey::ImGuiKey_F12));
        // unmapped like the Java host
        assert!(map_key("Minus").is_none());
        assert!(map_key("Numpad5").is_none());
        assert!(map_key("Fn").is_none());
    }
}
