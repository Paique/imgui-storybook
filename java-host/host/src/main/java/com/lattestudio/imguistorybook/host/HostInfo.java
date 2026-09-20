package com.lattestudio.imguistorybook.host;

import java.io.IOException;
import java.io.InputStream;
import java.util.Properties;

/** Build info baked into the jar by Gradle (host.properties). */
public final class HostInfo {

    private static final Properties PROPS = load();

    private HostInfo() {
    }

    public static String hostVersion() {
        return PROPS.getProperty("hostVersion", "dev");
    }

    public static String imguiJavaVersion() {
        return PROPS.getProperty("imguiJavaVersion", "unknown");
    }

    private static Properties load() {
        Properties props = new Properties();
        try (InputStream in = HostInfo.class.getResourceAsStream("/host.properties")) {
            if (in != null) {
                props.load(in);
            }
        } catch (IOException ignored) {
            // fall back to defaults
        }
        return props;
    }
}
