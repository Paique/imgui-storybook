//! Build info (Java: `HostInfo.java`, baked into the jar via Gradle `expand`).
//!
//! `BINDING_VERSION` mirrors the Dear ImGui version this binding tracks — it fills the
//! `imguiJavaVersion` protocol/catalog field so the TS layer stays host-agnostic.

/// Dear ImGui version tracked by the easy-imgui binding used at build time.
/// Keep in sync with the `easy-imgui` dependency (0.24.x tracks 1.92.9b).
pub const BINDING_VERSION: &str = "1.92.9.0";

pub const DEAR_IMGUI: &str = "1.92.9";

pub fn host_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn binding_version() -> &'static str {
    BINDING_VERSION
}
