package com.lattestudio.imguistorybook.host.gl;

import org.lwjgl.glfw.GLFWErrorCallback;

import java.nio.IntBuffer;

import static org.lwjgl.glfw.GLFW.GLFW_FALSE;
import static org.lwjgl.glfw.GLFW.GLFW_SCALE_TO_MONITOR;
import static org.lwjgl.glfw.GLFW.GLFW_VISIBLE;
import static org.lwjgl.glfw.GLFW.glfwCreateWindow;
import static org.lwjgl.glfw.GLFW.glfwDefaultWindowHints;
import static org.lwjgl.glfw.GLFW.glfwDestroyWindow;
import static org.lwjgl.glfw.GLFW.glfwInit;
import static org.lwjgl.glfw.GLFW.glfwMakeContextCurrent;
import static org.lwjgl.glfw.GLFW.glfwTerminate;
import static org.lwjgl.glfw.GLFW.glfwWindowHint;
import static org.lwjgl.opengl.GL11.GL_LINEAR;
import static org.lwjgl.opengl.GL11.GL_RGBA;
import static org.lwjgl.opengl.GL11.GL_RGBA8;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_2D;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_MAG_FILTER;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_MIN_FILTER;
import static org.lwjgl.opengl.GL11.GL_UNSIGNED_BYTE;
import static org.lwjgl.opengl.GL11.glBindTexture;
import static org.lwjgl.opengl.GL11.glGenTextures;
import static org.lwjgl.opengl.GL11.glTexImage2D;
import static org.lwjgl.opengl.GL11.glTexParameteri;
import static org.lwjgl.opengl.GL11.glViewport;
import static org.lwjgl.opengl.GL30.GL_COLOR_ATTACHMENT0;
import static org.lwjgl.opengl.GL30.GL_FRAMEBUFFER;
import static org.lwjgl.opengl.GL30.GL_FRAMEBUFFER_COMPLETE;
import static org.lwjgl.opengl.GL30.glBindFramebuffer;
import static org.lwjgl.opengl.GL30.glCheckFramebufferStatus;
import static org.lwjgl.opengl.GL30.glDeleteFramebuffers;
import static org.lwjgl.opengl.GL30.glFramebufferTexture2D;
import static org.lwjgl.opengl.GL30.glGenFramebuffers;
import static org.lwjgl.system.MemoryUtil.NULL;

/**
 * Hidden GLFW window providing an OpenGL context, plus an offscreen framebuffer object of exact
 * size. On Windows a hidden window reports a 0×0 framebuffer, and a visible one would be
 * DPI-scaled by the OS — rendering into our own FBO sidesteps both: the output is always
 * exactly {@code width × height} pixels on any machine. ImGui's DisplaySize is forced per frame
 * by ImGuiHost to match.
 */
public final class HeadlessGl implements AutoCloseable {

    private final long window;
    private final int width;
    private final int height;
    private int framebuffer;
    private int colorTexture;
    private final GLFWErrorCallback errorCallback;

    public HeadlessGl(int width, int height) {
        errorCallback = GLFWErrorCallback.createPrint(System.err).set();
        if (!glfwInit()) {
            throw new IllegalStateException("glfwInit failed");
        }
        glfwDefaultWindowHints();
        glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE);
        glfwWindowHint(GLFW_SCALE_TO_MONITOR, GLFW_FALSE);
        window = glfwCreateWindow(1, 1, "imgui-storybook host", NULL, NULL);
        if (window == NULL) {
            glfwTerminate();
            throw new IllegalStateException("glfwCreateWindow failed (no GL context?)");
        }
        glfwMakeContextCurrent(window);
        org.lwjgl.opengl.GL.createCapabilities();

        this.width = width;
        this.height = height;

        colorTexture = glGenTextures();
        glBindTexture(GL_TEXTURE_2D, colorTexture);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE,
                (java.nio.ByteBuffer) null);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

        framebuffer = glGenFramebuffers();
        glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);
        glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, colorTexture, 0);
        if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE) {
            throw new IllegalStateException("offscreen framebuffer incomplete");
        }
        glViewport(0, 0, width, height);
    }

    /** Logical size — always equals the framebuffer size (the FBO is exact). */
    public int width() {
        return width;
    }

    public int height() {
        return height;
    }

    public int framebufferWidth() {
        return width;
    }

    public int framebufferHeight() {
        return height;
    }

    public long window() {
        return window;
    }

    /** Re-binds the FBO and viewport (cheap safety if anything else rebound state). */
    public void bindFramebuffer() {
        glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);
        glViewport(0, 0, width, height);
    }

    @Override
    public void close() {
        glDeleteFramebuffers(framebuffer);
        org.lwjgl.opengl.GL11.glDeleteTextures(colorTexture);
        glfwDestroyWindow(window);
        glfwTerminate();
        errorCallback.free();
    }
}
