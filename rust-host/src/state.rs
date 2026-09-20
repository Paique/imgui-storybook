//! Current selection + view. Mutated from WS tasks under a mutex, snapshot-read by the render
//! thread (Java used volatile field swaps; a short mutex hold has the same observable
//! behavior and is simpler in Rust).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::api::{ArgSet, ArgValue, Backdrop, CanvasMode, Theme};
use crate::args_values::ArgMap;

#[derive(Clone, Debug)]
pub struct Inner {
    pub entry_index: Option<usize>,
    pub args: ArgMap,
    pub theme: Theme,
    pub backdrop: Backdrop,
    pub canvas_mode: CanvasMode,
    pub scale: f32,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            entry_index: None,
            args: ArgMap::new(),
            theme: Theme::Dark,
            backdrop: Backdrop::NeutralDark,
            canvas_mode: CanvasMode::Windowed,
            scale: 1.0,
        }
    }
}

#[derive(Default)]
pub struct ViewState {
    inner: Mutex<Inner>,
    /// Monotonic frame sequence number shared by the serve loop.
    frame_seq: AtomicU64,
}

impl ViewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("view state poisoned")
    }

    pub fn select(&self, entry_index: Option<usize>, arg_set: Option<&ArgSet>) {
        let mut g = self.lock();
        g.entry_index = entry_index;
        g.args = match (entry_index, arg_set) {
            (Some(_), Some(set)) => set.defaults(),
            _ => ArgMap::new(),
        };
    }

    pub fn set_args(&self, sanitized: ArgMap) {
        self.lock().args = sanitized;
    }

    /// Copy-on-write single arg update (used by `StoryContext.set`, drained after a frame).
    pub fn put_arg(&self, name: String, value: ArgValue) {
        self.lock().args.insert(name, value);
    }

    pub fn apply_sets(&self, sets: Vec<(String, ArgValue)>) {
        let mut g = self.lock();
        for (name, value) in sets {
            g.args.insert(name, value);
        }
    }

    /// Copy of the current state for the render thread (short lock, cheap clone).
    pub fn snapshot(&self) -> Inner {
        self.lock().clone()
    }

    pub fn next_frame_seq(&self) -> u64 {
        self.frame_seq.fetch_add(1, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ArgValue;

    #[test]
    fn select_resets_args() {
        let mut set = ArgSet::new();
        set.string("a", "x").bool("b", true);
        let state = ViewState::new();
        state.select(Some(0), Some(&set));
        {
            let g = state.lock();
            assert_eq!(g.entry_index, Some(0));
            assert_eq!(g.args.get("a"), Some(&ArgValue::Str("x".into())));
        }
        state.select(None, None);
        let g = state.lock();
        assert_eq!(g.entry_index, None);
        assert!(g.args.is_empty());
    }

    #[test]
    fn seq_is_monotonic() {
        let state = ViewState::new();
        assert_eq!(state.next_frame_seq(), 0);
        assert_eq!(state.next_frame_seq(), 1);
    }
}
