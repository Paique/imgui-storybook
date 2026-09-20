package com.lattestudio.imguistorybook.api;

import java.util.Locale;

/** Story id helpers. */
public final class StoryIds {

    private StoryIds() {
    }

    /**
     * Stable id for a story title: lowercase, non-alphanumeric runs collapsed to '-'.
     * {@code "Inputs/Text Input"} → {@code "inputs-text-input"}.
     */
    public static String slug(String title) {
        String slug = title.toLowerCase(Locale.ROOT)
                .replaceAll("[^a-z0-9]+", "-")
                .replaceAll("(^-+|-+$)", "");
        if (slug.isEmpty()) {
            throw new IllegalArgumentException("story title produces empty id: " + title);
        }
        return slug;
    }
}
