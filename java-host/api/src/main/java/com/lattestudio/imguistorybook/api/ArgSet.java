package com.lattestudio.imguistorybook.api;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Declaration of the editable args of a story, plus named presets (arg overrides).
 *
 * <p>Example:
 * <pre>{@code
 * args.string("label", "Confirmar")
 *     .bool("danger", false)
 *     .enumOf("status", Status.class, Status.PROGRESS)
 *     .preset("compacto", "label", "Ok");
 * }</pre>
 */
public final class ArgSet {

    private final Map<String, ArgSpec> specs = new LinkedHashMap<>();
    private final Map<String, Map<String, Object>> presets = new LinkedHashMap<>();

    public ArgSet string(String name, String defaultValue) {
        return add(new ArgSpec(name, ArgType.STRING, defaultValue, null, null));
    }

    public ArgSet bool(String name, boolean defaultValue) {
        return add(new ArgSpec(name, ArgType.BOOLEAN, defaultValue, null, null));
    }

    public ArgSet int32(String name, int defaultValue) {
        return add(new ArgSpec(name, ArgType.INT, defaultValue, null, null));
    }

    public ArgSet float32(String name, float defaultValue) {
        return add(new ArgSpec(name, ArgType.FLOAT, defaultValue, null, null));
    }

    public <T extends Enum<T>> ArgSet enumOf(String name, Class<T> type, T defaultValue) {
        @SuppressWarnings("unchecked")
        Class<? extends Enum<?>> raw = (Class<? extends Enum<?>>) type;
        T[] constants = type.getEnumConstants();
        String[] options = new String[constants.length];
        for (int i = 0; i < constants.length; i++) {
            options[i] = constants[i].name();
        }
        return add(new ArgSpec(name, ArgType.ENUM, defaultValue.name(), raw, options));
    }

    /** @param rgb color as {@code 0xRRGGBB} */
    public ArgSet color(String name, int rgb) {
        return add(new ArgSpec(name, ArgType.COLOR, rgb & 0xFFFFFF, null, null));
    }

    /**
     * Declares a named preset as a list of {@code name, value} pairs. Values not declared as args
     * are rejected at declaration time.
     */
    public ArgSet preset(String name, Object... nameValuePairs) {
        if (nameValuePairs.length % 2 != 0) {
            throw new IllegalArgumentException("preset '" + name + "': expected name/value pairs");
        }
        Map<String, Object> values = new LinkedHashMap<>();
        for (int i = 0; i < nameValuePairs.length; i += 2) {
            String argName = String.valueOf(nameValuePairs[i]);
            if (!specs.containsKey(argName)) {
                throw new IllegalArgumentException(
                        "preset '" + name + "': unknown arg '" + argName + "'");
            }
            values.put(argName, nameValuePairs[i + 1]);
        }
        presets.put(name, values);
        return this;
    }

    private ArgSet add(ArgSpec spec) {
        if (specs.containsKey(spec.name())) {
            throw new IllegalArgumentException("duplicate arg '" + spec.name() + "'");
        }
        specs.put(spec.name(), spec);
        return this;
    }

    public ArgSpec get(String name) {
        return specs.get(name);
    }

    public List<ArgSpec> specs() {
        return Collections.unmodifiableList(new ArrayList<>(specs.values()));
    }

    public Map<String, Map<String, Object>> presets() {
        return Collections.unmodifiableMap(presets);
    }

    /** Default values for all args, keyed by name. */
    public Map<String, Object> defaults() {
        Map<String, Object> out = new LinkedHashMap<>();
        specs.values().forEach(spec -> out.put(spec.name(), spec.defaultValue()));
        return out;
    }

    /** Default values overridden by the given preset. Unknown preset returns plain defaults. */
    public Map<String, Object> presetValues(String presetName) {
        Map<String, Object> out = defaults();
        Map<String, Object> overrides = presets.get(presetName);
        if (overrides != null) {
            out.putAll(overrides);
        }
        return out;
    }

    public boolean isEmpty() {
        return specs.isEmpty();
    }
}
