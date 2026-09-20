package com.lattestudio.imguistorybook.api;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.ServiceLoader;

/**
 * Registry of available stories. Combines stories discovered via {@link ServiceLoader} with
 * programmatically registered ones (used for the built-in demo stories).
 */
public final class StoryRegistry {

    public record Entry(String id, Story story, ArgSet args) {
    }

    private final Map<String, Entry> entries = new LinkedHashMap<>();

    public StoryRegistry add(Story story) {
        String id = StoryIds.slug(story.title());
        if (entries.containsKey(id)) {
            throw new IllegalArgumentException("duplicate story id '" + id + "' (title: "
                    + story.title() + ")");
        }
        ArgSet args = new ArgSet();
        story.defineArgs(args);
        entries.put(id, new Entry(id, story, args));
        return this;
    }

    public StoryRegistry addAll(List<Story> stories) {
        stories.forEach(this::add);
        return this;
    }

    /** Stories sorted by title (sidebar/docs order). */
    public List<Entry> entries() {
        List<Entry> out = new ArrayList<>(entries.values());
        out.sort(Comparator.comparing(e -> e.story().title(), String.CASE_INSENSITIVE_ORDER));
        return out;
    }

    public Optional<Entry> byId(String id) {
        return Optional.ofNullable(entries.get(id));
    }

    public int size() {
        return entries.size();
    }

    /** Discovers stories via {@link ServiceLoader} on the given classloader and returns a new registry. */
    public static StoryRegistry load(ClassLoader classLoader) {
        StoryRegistry registry = new StoryRegistry();
        ServiceLoader.load(Story.class, classLoader).forEach(registry::add);
        return registry;
    }
}
