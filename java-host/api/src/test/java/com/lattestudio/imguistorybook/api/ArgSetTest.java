package com.lattestudio.imguistorybook.api;

import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

class ArgSetTest {

    enum Status { OPEN, PROGRESS, CLOSED }

    @Test
    void declaresTypedArgsAndDefaults() {
        ArgSet args = new ArgSet()
                .string("label", "Confirmar")
                .bool("danger", true)
                .int32("count", 3)
                .float32("ratio", 0.5f)
                .enumOf("status", Status.class, Status.PROGRESS)
                .color("accent", 0xC28E5F);

        Map<String, Object> defaults = args.defaults();
        assertEquals(6, defaults.size());
        assertEquals("Confirmar", defaults.get("label"));
        assertEquals(true, defaults.get("danger"));
        assertEquals(3, defaults.get("count"));
        assertEquals(0.5f, defaults.get("ratio"));
        assertEquals("PROGRESS", defaults.get("status"));
        assertEquals(0xC28E5F, defaults.get("accent"));

        var spec = args.get("status");
        assertEquals(ArgType.ENUM, spec.type());
        assertArrayEquals(new String[]{"OPEN", "PROGRESS", "CLOSED"}, spec.enumOptions());
        assertEquals(ArgType.COLOR, args.get("accent").type());
    }

    @Test
    void presetsOverrideDefaultsAndKeepDeclarationOrder() {
        ArgSet args = new ArgSet()
                .string("label", "Confirmar")
                .bool("danger", false)
                .preset("danger", "label", "Excluir", "danger", true);

        Map<String, Object> preset = args.presetValues("danger");
        assertEquals("Excluir", preset.get("label"));
        assertEquals(true, preset.get("danger"));

        // unknown preset falls back to plain defaults
        assertEquals("Confirmar", args.presetValues("nope").get("label"));
        // declared order preserved
        assertEquals("label", args.specs().get(0).name());
    }

    @Test
    void rejectsDuplicateArg() {
        ArgSet args = new ArgSet().string("label", "a");
        assertThrows(IllegalArgumentException.class, () -> args.string("label", "b"));
    }

    @Test
    void rejectsPresetForUnknownArg() {
        ArgSet args = new ArgSet().string("label", "a");
        assertThrows(IllegalArgumentException.class, () -> args.preset("p", "nope", 1));
    }

    @Test
    void rejectsPresetWithOddPairs() {
        ArgSet args = new ArgSet().string("label", "a");
        assertThrows(IllegalArgumentException.class, () -> args.preset("p", "label"));
    }
}
