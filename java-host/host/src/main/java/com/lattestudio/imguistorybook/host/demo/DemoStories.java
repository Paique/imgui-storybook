package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.Story;

import java.util.List;

/** Built-in demo stories covering the standard ImGui widget vocabulary. */
public final class DemoStories {

    private DemoStories() {
    }

    public static List<Story> all() {
        return List.of(
                new ButtonStory(),
                new TextStory(),
                new InputTextStory(),
                new SliderStory(),
                new DragNumberStory(),
                new ComboSelectStory(),
                new CheckboxRadioStory(),
                new ColorPickerStory(),
                new TableStory(),
                new TabsStory(),
                new TreeStory(),
                new ProgressBarStory(),
                new TooltipStory(),
                new PopupModalStory(),
                new SelectableListStory(),
                new ChildWindowStory());
    }
}
