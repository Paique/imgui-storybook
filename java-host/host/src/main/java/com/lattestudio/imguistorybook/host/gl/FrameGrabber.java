package com.lattestudio.imguistorybook.host.gl;

import org.lwjgl.stb.STBIWriteCallback;
import org.lwjgl.stb.STBImageWrite;
import org.lwjgl.system.MemoryUtil;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.lwjgl.BufferUtils.createByteBuffer;
import static org.lwjgl.opengl.GL11.GL_PACK_ALIGNMENT;
import static org.lwjgl.opengl.GL11.GL_RGBA;
import static org.lwjgl.opengl.GL11.GL_UNSIGNED_BYTE;
import static org.lwjgl.opengl.GL11.glPixelStorei;
import static org.lwjgl.opengl.GL11.glReadPixels;

/**
 * Reads the offscreen framebuffer as flipped RGBA (reusable buffer) and encodes JPEG/PNG with
 * stb_image_write. ImageIO was profiled at ~100ms per JPEG at 900×600 — stb keeps the live
 * stream at the 30fps budget. Alpha is forced opaque so PNG captures never show through.
 */
public final class FrameGrabber {

    private static final int RGBA = 4;

    private ByteBuffer buffer;

    /**
     * Reads the current framebuffer into a reusable RGBA buffer, flipped vertically (GL origin is
     * bottom-left) with alpha forced to 0xFF.
     */
    public ByteBuffer grab(int width, int height) {
        int size = width * height * RGBA;
        if (buffer == null || buffer.capacity() < size) {
            buffer = createByteBuffer(size);
        }
        buffer.clear();
        glPixelStorei(GL_PACK_ALIGNMENT, 1);
        glReadPixels(0, 0, width, height, GL_RGBA, GL_UNSIGNED_BYTE, buffer);
        flipAndOpaque(buffer, width, height);
        return buffer;
    }

    /** @param quality 0..100 */
    public byte[] jpeg(ByteBuffer rgba, int width, int height, int quality) {
        return encode("jpeg", (out, callback) ->
                STBImageWrite.stbi_write_jpg_to_func(callback, 0L, width, height, RGBA, rgba, quality),
                width, height);
    }

    public void png(ByteBuffer rgba, int width, int height, Path file) {
        byte[] bytes = encode("png", (out, callback) ->
                STBImageWrite.stbi_write_png_to_func(callback, 0L, width, height, RGBA, rgba, width * RGBA),
                width, height);
        try {
            Files.createDirectories(file.getParent());
            Files.write(file, bytes);
        } catch (IOException e) {
            throw new IllegalStateException("PNG write failed: " + file, e);
        }
    }

    private interface EncodeCall {
        boolean run(ByteArrayOutputStream out, STBIWriteCallback callback);
    }

    private byte[] encode(String format, EncodeCall call, int width, int height) {
        ByteArrayOutputStream out = new ByteArrayOutputStream(96 * 1024);
        try (STBIWriteCallback callback = new STBIWriteCallback() {
            @Override
            public void invoke(long context, long data, int size) {
                ByteBuffer chunk = MemoryUtil.memByteBuffer(data, size);
                byte[] bytes = new byte[size];
                chunk.get(bytes);
                out.write(bytes, 0, size);
            }
        }) {
            rgba.position(0);
            rgba.limit(width * height * RGBA);
            if (!call.run(out, callback)) {
                throw new IllegalStateException(format + " encode failed");
            }
        }
        return out.toByteArray();
    }

    private static void flipAndOpaque(ByteBuffer buffer, int width, int height) {
        int rowBytes = width * RGBA;
        byte[] top = new byte[rowBytes];
        byte[] bottom = new byte[rowBytes];
        for (int y = 0, mirror = height - 1; y < mirror; y++, mirror--) {
            int topPos = y * rowBytes;
            int bottomPos = mirror * rowBytes;
            buffer.get(topPos, top, 0, rowBytes);
            buffer.get(bottomPos, bottom, 0, rowBytes);
            buffer.put(topPos, bottom, 0, rowBytes);
            buffer.put(bottomPos, top, 0, rowBytes);
        }
        for (int i = 3; i < width * height * RGBA; i += RGBA) {
            buffer.put(i, (byte) 0xFF);
        }
    }
}
