package com.lattestudio.imguistorybook.api;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class StoryRegistryTest {

    static class FakeStory implements Story {
        private final String title;

        FakeStory(String title) {
            this.title = title;
        }

        @Override
        public String title() {
            return title;
        }

        @Override
        public void render(StoryContext ctx) {
        }
    }

    @Test
    void slugIsStableAndLowercase() {
        assertEquals("inputs-text-input", StoryIds.slug("Inputs/Text Input"));
        assertEquals("button", StoryIds.slug("  Button  "));
        assertThrows(IllegalArgumentException.class, () -> StoryIds.slug("///"));
    }

    @Test
    void entriesSortedByTitleCaseInsensitive() {
        StoryRegistry registry = new StoryRegistry()
                .add(new FakeStory("Inputs/Zebra"))
                .add(new FakeStory("inputs/apple"))
                .add(new FakeStory("Basics/Button"));

        List<StoryRegistry.Entry> entries = registry.entries();
        assertEquals(3, entries.size());
        assertEquals("Basics/Button", entries.get(0).story().title());
        assertEquals("inputs/apple", entries.get(1).story().title());
        assertEquals("Inputs/Zebra", entries.get(2).story().title());
        // ids are unique per slug (case-insensitive collapse)
        assertEquals("inputs-apple", entries.get(1).id());
    }

    @Test
    void duplicateSlugRejected() {
        StoryRegistry registry = new StoryRegistry().add(new FakeStory("Inputs/Text Input"));
        assertThrows(IllegalArgumentException.class,
                () -> registry.add(new FakeStory("inputs text input")));
    }

    @Test
    void byIdReturnsEntryWithArgSet() {
        Story story = new FakeStory("Basics/Button") {
            @Override
            public void defineArgs(ArgSet args) {
                args.string("label", "Go");
            }
        };
        StoryRegistry registry = new StoryRegistry().add(story);

        var entry = registry.byId("basics-button").orElseThrow();
        assertSame(story, entry.story());
        assertEquals("Go", entry.args().get("label").defaultValue());
        assertTrue(registry.byId("missing").isEmpty());
    }
}
