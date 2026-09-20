//! Input queue: WS tasks enqueue synthetic events, the render thread drains them into
//! `ImGuiIO` right before `ImGui::new_frame`. Rust mirror of `host/net/InputInjector.java`
//! (the io event application lives in `imgui_host.rs`, where the io handle is available).

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::protocol::InputEvent;

#[derive(Default)]
pub struct InputInjector {
    queue: Mutex<VecDeque<InputEvent>>,
}

impl InputInjector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, ev: InputEvent) {
        self.queue.lock().expect("input queue").push_back(ev);
    }

    /// Drains all pending events (render thread, once per frame).
    pub fn drain(&self) -> Vec<InputEvent> {
        let mut q = self.queue.lock().expect("input queue");
        q.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_drain() {
        let inj = InputInjector::new();
        inj.push(InputEvent::MouseMove(1.0, 2.0));
        inj.push(InputEvent::Text("a".into()));
        assert_eq!(
            inj.drain(),
            vec![
                InputEvent::MouseMove(1.0, 2.0),
                InputEvent::Text("a".into())
            ]
        );
        assert!(inj.drain().is_empty());
    }
}
