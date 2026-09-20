package com.lattestudio.imguistorybook.host.core;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.ArgSpec;
import com.lattestudio.imguistorybook.api.ArgType;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Coercion between JSON-ish values (Gson: Double/Boolean/String) coming from the web client and
 * the canonical internal representation of each {@link ArgType}.
 *
 * <p>Canonical forms: STRING → String, BOOLEAN → Boolean, INT → Integer, FLOAT → Float,
 * ENUM → option name String, COLOR → Integer {@code 0xRRGGBB}.
 */
public final class ArgValues {

    private ArgValues() {
    }

    /** Typed read with fallback to the declared default when missing/invalid. */
    public static Object get(ArgSpec spec, Map<String, Object> values) {
        if (!values.containsKey(spec.name())) {
            return spec.defaultValue();
        }
        Object coerced = coerce(spec, values.get(spec.name()));
        return coerced != null ? coerced : spec.defaultValue();
    }

    public static String getString(ArgSpec spec, Map<String, Object> values) {
        return String.valueOf(get(spec, values));
    }

    public static boolean getBoolean(ArgSpec spec, Map<String, Object> values) {
        Object value = get(spec, values);
        return value instanceof Boolean b ? b : Boolean.parseBoolean(String.valueOf(value));
    }

    public static int getInt(ArgSpec spec, Map<String, Object> values) {
        Object value = get(spec, values);
        if (value instanceof Number n) {
            return n.intValue();
        }
        try {
            return (int) Math.rint(Double.parseDouble(String.valueOf(value)));
        } catch (NumberFormatException e) {
            return (Integer) spec.defaultValue();
        }
    }

    public static float getFloat(ArgSpec spec, Map<String, Object> values) {
        Object value = get(spec, values);
        if (value instanceof Number n) {
            return n.floatValue();
        }
        try {
            return Float.parseFloat(String.valueOf(value));
        } catch (NumberFormatException e) {
            return (Float) spec.defaultValue();
        }
    }

    /** @return {@code 0xRRGGBB} */
    public static int getColor(ArgSpec spec, Map<String, Object> values) {
        Object value = get(spec, values);
        if (value instanceof Number n) {
            return n.intValue() & 0xFFFFFF;
        }
        String s = String.valueOf(value);
        try {
            String hex = s.startsWith("#") ? s.substring(1) : s;
            return Integer.parseInt(hex, 16) & 0xFFFFFF;
        } catch (RuntimeException e) {
            return (Integer) spec.defaultValue() & 0xFFFFFF;
        }
    }

    /** Coerces one value to the canonical form, or null when impossible. */
    public static Object coerce(ArgSpec spec, Object raw) {
        if (raw == null) {
            return null;
        }
        return switch (spec.type()) {
            case STRING -> raw instanceof String s ? s : String.valueOf(raw);
            case BOOLEAN -> raw instanceof Boolean b ? b : Boolean.parseBoolean(String.valueOf(raw));
            case INT -> raw instanceof Number n ? n.intValue()
                    : parseDoubleOrNull(String.valueOf(raw), d -> (int) Math.rint(d));
            case FLOAT -> raw instanceof Number n ? n.floatValue()
                    : parseDoubleOrNull(String.valueOf(raw), Number::floatValue);
            case ENUM -> {
                String name = String.valueOf(raw);
                for (String option : spec.enumOptions()) {
                    if (option.equals(name)) {
                        yield name;
                    }
                }
                yield null;
            }
            case COLOR -> raw instanceof Number n ? n.intValue() & 0xFFFFFF
                    : parseHexColor(String.valueOf(raw));
        };
    }

    /** Keeps only known args (coerced); used to sanitize client input. */
    public static Map<String, Object> sanitize(ArgSet set, Map<String, Object> raw) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (ArgSpec spec : set.specs()) {
            Object value = raw.get(spec.name());
            if (value != null) {
                Object coerced = coerce(spec, value);
                if (coerced != null) {
                    out.put(spec.name(), coerced);
                }
            }
        }
        return out;
    }

    /** Full map coerced against the schema (missing → defaults). */
    public static Map<String, Object> materialize(ArgSet set, Map<String, Object> raw) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (ArgSpec spec : set.specs()) {
            out.put(spec.name(), get(spec, raw));
        }
        return out;
    }

    /** JSON-friendly form: colors as {@code #RRGGBB}, everything else natural. */
    public static Map<String, Object> forJson(Map<String, Object> values, ArgSet set) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (ArgSpec spec : set.specs()) {
            Object value = values.get(spec.name());
            if (value == null) {
                continue;
            }
            if (spec.type() == ArgType.COLOR) {
                out.put(spec.name(), String.format("#%06X", (Integer) value & 0xFFFFFF));
            } else {
                out.put(spec.name(), value);
            }
        }
        return out;
    }

    private static Integer parseHexColor(String s) {
        try {
            String hex = s.startsWith("#") ? s.substring(1) : s;
            return Integer.parseInt(hex, 16) & 0xFFFFFF;
        } catch (RuntimeException e) {
            return null;
        }
    }

    private interface Parse<T> {
        T apply(Double d);
    }

    private static <T> T parseDoubleOrNull(String s, Parse<T> then) {
        try {
            return then.apply(Double.parseDouble(s));
        } catch (NumberFormatException e) {
            return null;
        }
    }
}
