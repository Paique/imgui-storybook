//! Small sys-level helpers for widgets the high-level `Ui` doesn't wrap. Each takes/returns
//! plain Rust types and handles the CString dance. All must run inside the ImGui frame.

use std::ffi::CString;

use easy_imgui_sys as sys;

pub(crate) fn c(s: &str) -> CString {
    CString::new(s).unwrap_or_default()
}

// --- colors ---

pub(crate) fn push_style_color(idx: sys::ImGuiCol, rgba: [f32; 4]) {
    let v = sys::ImVec4 {
        x: rgba[0],
        y: rgba[1],
        z: rgba[2],
        w: rgba[3],
    };
    unsafe { sys::ImGui_PushStyleColor1(idx, &v) };
}

pub(crate) fn pop_style_color(count: i32) {
    unsafe { sys::ImGui_PopStyleColor(count) };
}

// --- disabled ---

pub(crate) fn begin_disabled(disabled: bool) {
    unsafe { sys::ImGui_BeginDisabled(disabled) };
}

pub(crate) fn end_disabled() {
    unsafe { sys::ImGui_EndDisabled() };
}

// --- sliders / drags (no high-level wrapper in easy-imgui 0.24) ---

pub(crate) fn slider_float(label: &str, v: &mut f32, min: f32, max: f32, format: &str) -> bool {
    let l = c(label);
    let f = c(format);
    unsafe {
        sys::ImGui_SliderFloat(l.as_ptr(), v, min, max, f.as_ptr(), sys::ImGuiSliderFlags_::ImGuiSliderFlags_None.0)
    }
}

pub(crate) fn drag_float(label: &str, v: &mut f32, speed: f32, format: &str) -> bool {
    let l = c(label);
    let f = c(format);
    unsafe {
        sys::ImGui_DragFloat(
            l.as_ptr(),
            v,
            speed,
            0.0,
            0.0,
            f.as_ptr(),
            sys::ImGuiSliderFlags_::ImGuiSliderFlags_None.0,
        )
    }
}

pub(crate) fn drag_int(label: &str, v: &mut i32, speed: f32, min: i32, max: i32) -> bool {
    let l = c(label);
    unsafe { sys::ImGui_DragInt(l.as_ptr(), v, speed, min, max, c"%d".as_ptr(), 0) }
}

// --- colors ---

pub(crate) fn color_edit3(label: &str, color: &mut [f32; 3]) -> bool {
    let l = c(label);
    unsafe { sys::ImGui_ColorEdit3(l.as_ptr(), color.as_mut_ptr(), 0) }
}

pub(crate) fn color_picker3(label: &str, color: &mut [f32; 3]) -> bool {
    let l = c(label);
    unsafe { sys::ImGui_ColorPicker3(l.as_ptr(), color.as_mut_ptr(), 0) }
}

// --- progress ---

pub(crate) fn progress_bar(fraction: f32, overlay: &str) {
    let o = c(overlay);
    let size = sys::ImVec2 { x: -1.0, y: 0.0 };
    unsafe { sys::ImGui_ProgressBar(fraction, &size, o.as_ptr()) };
}

// --- tooltip ---

pub(crate) fn set_tooltip(text: &str) {
    let t = c(text);
    unsafe { sys::ImGui_SetTooltip(t.as_ptr()) };
}

// --- tabs ---

pub(crate) fn begin_tab_bar(id: &str) -> bool {
    let s = c(id);
    unsafe { sys::ImGui_BeginTabBar(s.as_ptr(), sys::ImGuiTabBarFlags_::ImGuiTabBarFlags_None.0) }
}

pub(crate) fn end_tab_bar() {
    unsafe { sys::ImGui_EndTabBar() };
}

pub(crate) fn begin_tab_item(label: &str) -> bool {
    let l = c(label);
    unsafe { sys::ImGui_BeginTabItem(l.as_ptr(), std::ptr::null_mut(), 0) }
}

pub(crate) fn end_tab_item() {
    unsafe { sys::ImGui_EndTabItem() };
}

// --- tables ---

pub(crate) fn table_flags_borders() -> i32 {
    sys::ImGuiTableFlags_::ImGuiTableFlags_Borders.0
}

pub(crate) fn table_flags_row_bg() -> i32 {
    sys::ImGuiTableFlags_::ImGuiTableFlags_RowBg.0
}

pub(crate) fn begin_table(id: &str, columns: i32, flags: i32) -> bool {
    let s = c(id);
    let size = sys::ImVec2 { x: 0.0, y: 0.0 };
    unsafe { sys::ImGui_BeginTable(s.as_ptr(), columns, flags, &size, 0.0) }
}

pub(crate) fn end_table() {
    unsafe { sys::ImGui_EndTable() };
}

pub(crate) fn table_setup_column(label: &str) {
    let l = c(label);
    unsafe { sys::ImGui_TableSetupColumn(l.as_ptr(), 0, 0.0, 0) };
}

pub(crate) fn table_headers_row() {
    unsafe { sys::ImGui_TableHeadersRow() };
}

pub(crate) fn table_next_row() {
    unsafe { sys::ImGui_TableNextRow(0, 0.0) };
}

pub(crate) fn table_next_column() {
    unsafe { sys::ImGui_TableNextColumn() };
}

// --- modal ---

pub(crate) fn begin_popup_modal(name: &str) -> bool {
    let n = c(name);
    unsafe { sys::ImGui_BeginPopupModal(n.as_ptr(), std::ptr::null_mut(), 0) }
}

pub(crate) fn end_popup() {
    unsafe { sys::ImGui_EndPopup() };
}

// --- child region (1.92 signature: (str_id, size, child_flags, window_flags)) ---

pub(crate) fn begin_child(id: &str, height: f32, border: bool) -> bool {
    let s = c(id);
    let size = sys::ImVec2 { x: 0.0, y: height };
    unsafe {
        sys::ImGui_BeginChild(
            s.as_ptr(),
            &size,
            if border {
                sys::ImGuiChildFlags_::ImGuiChildFlags_Borders.0
            } else {
                sys::ImGuiChildFlags_::ImGuiChildFlags_None.0
            },
            0,
        )
    }
}

pub(crate) fn end_child() {
    unsafe { sys::ImGui_EndChild() };
}
