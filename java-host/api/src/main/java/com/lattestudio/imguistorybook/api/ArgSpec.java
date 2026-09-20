package com.lattestudio.imguistorybook.api;

import java.util.Locale;

/**
 * Immutable declaration of one arg: name, type, default value and (for enums) the option names.
 */
public final class ArgSpec {

    private final String name;
    private final ArgType type;
    private final Object defaultValue;
    private final Class<? extends Enum<?>> enumType;
    private final String[] enumOptions;

    ArgSpec(String name, ArgType type, Object defaultValue,
            Class<? extends Enum<?>> enumType, String[] enumOptions) {
        this.name = name;
        this.type = type;
        this.defaultValue = defaultValue;
        this.enumType = enumType;
        this.enumOptions = enumOptions;
    }

    public String name() {
        return name;
    }

    public ArgType type() {
        return type;
    }

    public Object defaultValue() {
        return defaultValue;
    }

    public Class<? extends Enum<?>> enumType() {
        return enumType;
    }

    public String[] enumOptions() {
        return enumOptions;
    }

    /** Human-readable default, used in generated docs. */
    public String defaultAsString() {
        return switch (type) {
            case COLOR -> String.format("#%06X", (Integer) defaultValue);
            case FLOAT -> trimFloat((Float) defaultValue);
            case INT -> String.valueOf(defaultValue);
            case BOOLEAN -> String.valueOf(defaultValue);
            default -> String.valueOf(defaultValue);
        };
    }

    private static String trimFloat(float v) {
        if (v == Math.rint(v)) {
            return String.valueOf((long) v);
        }
        return String.format(Locale.ROOT, "%.3f", v).replaceAll("0+$", "").replaceAll("\\.$", "");
    }
}
