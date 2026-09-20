//! Registry of available stories. Rust mirror of `api/StoryRegistry.java`.

use crate::api::{story_slug, ArgSet, Story};

/// Immutable metadata + declared args of a registered story (shared with the WS threads).
#[derive(Clone, Debug)]
pub struct StoryMeta {
    pub id: String,
    pub title: String,
    pub description: String,
    /// Type path of the story implementation (Java used the FQ class name).
    pub story_class: String,
    pub args: ArgSet,
}

#[derive(Debug)]
pub struct Entry {
    pub meta: StoryMeta,
    /// Index into the registry's story list (stories live on the render thread).
    pub index: usize,
}

#[derive(Default)]
pub struct StoryRegistry {
    stories: Vec<Box<dyn Story>>,
    entries: Vec<Entry>,
}

impl StoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<S: Story + 'static>(&mut self, story: S) -> &mut Self {
        let title = story.title().to_string();
        let id = story_slug(&title);
        assert!(
            !self.entries.iter().any(|e| e.meta.id == id),
            "duplicate story id '{id}' (title: {title})"
        );
        let mut args = ArgSet::new();
        story.define_args(&mut args);
        let story_class = trim_type_path(std::any::type_name::<S>());
        let index = self.stories.len();
        self.stories.push(Box::new(story));
        let description = self.stories[index].description().to_string();
        self.entries.push(Entry {
            meta: StoryMeta {
                id,
                title,
                description,
                story_class,
                args,
            },
            index,
        });
        self
    }

    pub fn add_all<S, I>(&mut self, stories: I) -> &mut Self
    where
        S: Story + 'static,
        I: IntoIterator<Item = S>,
    {
        for story in stories {
            self.add(story);
        }
        self
    }

    /// Stories sorted by title (sidebar/docs order), case-insensitive.
    pub fn entries(&self) -> Vec<&Entry> {
        let mut out: Vec<&Entry> = self.entries.iter().collect();
        out.sort_by(|a, b| a.meta.title.to_lowercase().cmp(&b.meta.title.to_lowercase()));
        out
    }

    pub fn by_id(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.meta.id == id)
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Borrow a story by registry index (render thread only).
    pub fn story_mut(&mut self, index: usize) -> &mut dyn Story {
        self.stories[index].as_mut()
    }
}

/// Type path of a story trimmed to the last two segments (`demo::button::ButtonStory`).
fn trim_type_path(full: &str) -> String {
    let mut parts: Vec<&str> = full.rsplit("::").take(2).collect();
    parts.reverse();
    parts.join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::StoryCtx;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct Fake {
        title: String,
    }

    impl Story for Fake {
        fn title(&self) -> &str {
            &self.title
        }

        fn define_args(&self, args: &mut ArgSet) {
            args.string("x", "y");
        }

        fn render(&mut self, _ui: &crate::imgui_host::StoryUi, _ctx: &mut StoryCtx) {}
    }

    fn fake(title: &str) -> Fake {
        Fake {
            title: title.to_string(),
        }
    }

    #[test]
    fn add_slug_dedup_and_order() {
        let mut reg = StoryRegistry::new();
        reg.add(fake("Basics/Zeta")).add(fake("Basics/Alpha"));
        let ids: Vec<String> = reg.entries().iter().map(|e| e.meta.id.clone()).collect();
        assert_eq!(ids, ["basics-alpha", "basics-zeta"]);
        assert!(reg.by_id("basics-zeta").is_some());
        assert!(reg.by_id("nope").is_none());
        assert_eq!(reg.size(), 2);
    }

    #[test]
    #[should_panic(expected = "duplicate story id")]
    fn duplicate_id_panics() {
        let mut reg = StoryRegistry::new();
        reg.add(fake("Same Title")).add(fake("Same   Title"));
    }

    #[test]
    fn define_args_called_once() {
        let mut reg = StoryRegistry::new();
        reg.add(fake("Arged"));
        let entry = reg.by_id("arged").unwrap();
        assert_eq!(entry.meta.args.specs().len(), 1);
        assert_eq!(entry.meta.args.specs()[0].name, "x");
        let _ = COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}
