package com.lattestudio.imguistorybook.host.core;

import com.lattestudio.imguistorybook.api.ArgSpec;
import com.lattestudio.imguistorybook.api.StoryContext;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.Theme;

import java.util.Map;

/** StoryContext implementation bound to the current frame's entry/args/view. */
public final class HostedStoryContext implements StoryContext {

    private final ImGuiHost host;
    private final ViewState state;
    private StoryRegistry.Entry entry;
    private Map<String, Object> args;
    private ViewState view;

    public HostedStoryContext(ImGuiHost host, ViewState state) {
        this.host = host;
        this.state = state;
    }

    void update(StoryRegistry.Entry entry, Map<String, Object> args, ViewState view) {
        this.entry = entry;
        this.args = args;
        this.view = view;
    }

    @Override
    public String string(String name) {
        ArgSpec spec = spec(name);
        return spec == null ? "" : ArgValues.getString(spec, args);
    }

    @Override
    public boolean bool(String name) {
        ArgSpec spec = spec(name);
        return spec != null && ArgValues.getBoolean(spec, args);
    }

    @Override
    public int int32(String name) {
        ArgSpec spec = spec(name);
        return spec == null ? 0 : ArgValues.getInt(spec, args);
    }

    @Override
    public float float32(String name) {
        ArgSpec spec = spec(name);
        return spec == null ? 0f : ArgValues.getFloat(spec, args);
    }

    @Override
    public <T extends Enum<T>> T enumValue(String name, Class<T> type) {
        ArgSpec spec = spec(name);
        if (spec == null) {
            return null;
        }
        String value = ArgValues.getString(spec, args);
        try {
            return Enum.valueOf(type, value);
        } catch (IllegalArgumentException e) {
            T[] constants = type.getEnumConstants();
            for (T constant : constants) {
                if (constant.name().equals(spec.defaultValue())) {
                    return constant;
                }
            }
            return constants.length > 0 ? constants[0] : null;
        }
    }

    @Override
    public int color(String name) {
        ArgSpec spec = spec(name);
        return spec == null ? 0xFFFFFF : ArgValues.getColor(spec, args);
    }

    @Override
    public Theme theme() {
        return view == null ? Theme.DARK : view.theme();
    }

    @Override
    public float scale() {
        return view == null ? 1f : view.scale();
    }

    @Override
    public void action(String name) {
        host.recordAction(name);
    }

    @Override
    public void set(String name, Object value) {
        ArgSpec spec = spec(name);
        if (spec == null || entry == null) {
            return;
        }
        Object coerced = ArgValues.coerce(spec, value);
        if (coerced != null) {
            state.putArg(entry.args(), name, coerced);
        }
    }

    private ArgSpec spec(String name) {
        return entry == null ? null : entry.args().get(name);
    }
}
