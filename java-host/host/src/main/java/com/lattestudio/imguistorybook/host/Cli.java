package com.lattestudio.imguistorybook.host;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;

/** Minimal hand-rolled CLI parser (no dependencies). */
public final class Cli {

    public enum Command { LIST, SERVE, CAPTURE }

    private Command command = Command.LIST;
    private boolean json = false;
    private boolean noDemo = false;
    private int port = 8765;
    private int width = 900;
    private int height = 600;
    private String out = "captures";
    private List<String> themes = List.of("light", "dark");
    private List<String> scales = List.of("1");

    public static Cli parse(String[] args) {
        Cli cli = new Cli();
        for (int i = 0; i < args.length; i++) {
            String arg = args[i];
            switch (arg) {
                case "--list" -> cli.command = Command.LIST;
                case "--json" -> cli.json = true;
                case "--serve" -> cli.command = Command.SERVE;
                case "--capture" -> cli.command = Command.CAPTURE;
                case "--no-demo" -> cli.noDemo = true;
                case "--port" -> cli.port = Integer.parseInt(next(args, arg, ++i));
                case "--width" -> cli.width = Integer.parseInt(next(args, arg, ++i));
                case "--height" -> cli.height = Integer.parseInt(next(args, arg, ++i));
                case "--out" -> cli.out = next(args, arg, ++i);
                case "--themes" -> cli.themes = split(next(args, arg, ++i));
                case "--scales" -> cli.scales = split(next(args, arg, ++i));
                default -> throw new IllegalArgumentException("unknown argument: " + arg);
            }
        }
        return cli;
    }

    private static String next(String[] args, String flag, int index) {
        if (index >= args.length) {
            throw new IllegalArgumentException(flag + " expects a value");
        }
        return args[index];
    }

    private static List<String> split(String value) {
        List<String> out = new ArrayList<>();
        for (String part : value.split(",")) {
            String trimmed = part.trim().toLowerCase(Locale.ROOT);
            if (!trimmed.isEmpty()) {
                out.add(trimmed);
            }
        }
        return out;
    }

    public Command command() {
        return command;
    }

    public boolean json() {
        return json;
    }

    public boolean noDemo() {
        return noDemo;
    }

    public int port() {
        return port;
    }

    public int width() {
        return width;
    }

    public int height() {
        return height;
    }

    public String out() {
        return out;
    }

    public List<String> themes() {
        return themes;
    }

    public List<String> scales() {
        return scales;
    }

    public static String usage() {
        return """
                imgui-storybook java host

                Usage:
                  host --list --json
                  host --serve [--port 8765] [--width 900] [--height 600] [--no-demo]
                  host --capture [--out DIR] [--themes light,dark] [--scales 1,2] \
                       [--width 900] [--height 600] [--no-demo]

                Stories from other projects are discovered via
                META-INF/services/com.lattestudio.imguistorybook.api.Story on the classpath.
                """;
    }
}
