package dev.betterendfield.next;

import android.content.Context;
import android.content.SharedPreferences;
import android.opengl.EGL14;
import android.opengl.EGLConfig;
import android.opengl.EGLContext;
import android.opengl.EGLDisplay;
import android.opengl.EGLSurface;
import android.opengl.GLES20;
import android.os.Build;

/**
 * Read-only texture capability probe on the installation worker, never the game GL context.
 * The game backend still re-checks every format through SystemInfo.SupportsTextureFormat;
 * this probe only decides whether an import needs a texture rewrite.
 */
final class TextureSupport {
    static final int RGBA32 = 4, BC1 = 10, BC3 = 12, BC7 = 25, BC4 = 26, BC5 = 27;
    static final int ETC2_RGB = 45, ETC2_RGBA1 = 46, ETC2_RGBA8 = 47;
    static final int ASTC_4X4 = 48, ASTC_5X5 = 49, ASTC_6X6 = 50;
    static final int R8 = 63;

    private static final String PROFILE_PREFIX = "texture_profile_v1";

    static final class Profile {
        final boolean astc, bc1, bc3, bc4, bc5, bc7, etc2;
        final int maxTextureSize;

        Profile(boolean astc, boolean bc1, boolean bc3, boolean bc4, boolean bc5, boolean bc7,
                boolean etc2, int maxTextureSize) {
            this.astc = astc; this.bc1 = bc1; this.bc3 = bc3; this.bc4 = bc4; this.bc5 = bc5;
            this.bc7 = bc7; this.etc2 = etc2; this.maxTextureSize = maxTextureSize;
        }

        /**
         * True when the BEM runtime can upload this format on this device. ETC2 is reported by
         * the probe but not accepted by the current BEM reader/runtime, so it must be rewritten.
         */
        boolean supports(int format) {
            switch (format) {
                case RGBA32: case R8: return true; // Core on every backend the game runs on.
                case BC1: return bc1;
                case BC3: return bc3;
                case BC4: return bc4;
                case BC5: return bc5;
                case BC7: return bc7;
                case ETC2_RGB: case ETC2_RGBA1: case ETC2_RGBA8: return false; // runtime pending
                case ASTC_4X4: case ASTC_5X5: case ASTC_6X6: return astc;
                default: return false;
            }
        }

        /** Bit mask understood by the native texture converter (per-texture keep/skip). */
        int supportedMask() {
            return (bc1 ? 1 : 0) | (bc3 ? 2 : 0) | (bc4 ? 4 : 0) | (bc5 ? 8 : 0)
                    | (bc7 ? 16 : 0) | (astc ? 32 : 0);
        }

        String describe() {
            return "astc=" + (astc ? 1 : 0)
                    + " etc2=" + (etc2 ? 1 : 0)
                    + " bc1=" + (bc1 ? 1 : 0) + " bc3=" + (bc3 ? 1 : 0)
                    + " bc4=" + (bc4 ? 1 : 0) + " bc5=" + (bc5 ? 1 : 0)
                    + " bc7=" + (bc7 ? 1 : 0) + " max=" + maxTextureSize;
        }
    }

    /** Cached probe: per device/ROM, the EGL capability set only changes with a driver update. */
    static Profile probe(Context context) {
        Context app = context.getApplicationContext();
        final String key = PROFILE_PREFIX + "::" + Build.FINGERPRINT + "::" + Build.VERSION.SDK_INT;
        SharedPreferences preferences = SettingsStore.open(app);
        String stored = preferences.getString(key, null);
        if (stored != null) {
            Profile cached = parse(stored);
            if (cached != null) return cached;
        }
        Profile profile = probe();
        if (profile.maxTextureSize > 0) {
            preferences.edit().putString(key, encode(profile)).apply();
        }
        return profile;
    }

    /** Raw probe (creates its own EGL context; keep off the main thread). */
    static Profile probe() {
        EGLDisplay display = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY);
        if (display == EGL14.EGL_NO_DISPLAY) return unavailable();
        EGLContext context = EGL14.EGL_NO_CONTEXT;
        EGLSurface surface = EGL14.EGL_NO_SURFACE;
        try {
            if (!EGL14.eglInitialize(display, new int[2], 0, new int[2], 0)) return unavailable();
            EGLConfig[] configs = new EGLConfig[1];
            int[] count = new int[1];
            int[] attributes = {EGL14.EGL_RENDERABLE_TYPE, 4, EGL14.EGL_SURFACE_TYPE,
                    EGL14.EGL_PBUFFER_BIT, EGL14.EGL_NONE};
            if (!EGL14.eglChooseConfig(display, attributes, 0, configs, 0, 1, count, 0)
                    || count[0] == 0) return unavailable();
            context = EGL14.eglCreateContext(display, configs[0], EGL14.EGL_NO_CONTEXT,
                    new int[]{EGL14.EGL_CONTEXT_CLIENT_VERSION, 2, EGL14.EGL_NONE}, 0);
            surface = EGL14.eglCreatePbufferSurface(display, configs[0],
                    new int[]{EGL14.EGL_WIDTH, 1, EGL14.EGL_HEIGHT, 1, EGL14.EGL_NONE}, 0);
            if (context == EGL14.EGL_NO_CONTEXT || surface == EGL14.EGL_NO_SURFACE
                    || !EGL14.eglMakeCurrent(display, surface, surface, context)) return unavailable();
            String extensions = GLES20.glGetString(GLES20.GL_EXTENSIONS);
            String version = GLES20.glGetString(GLES20.GL_VERSION);
            int[] maximum = new int[1];
            GLES20.glGetIntegerv(GLES20.GL_MAX_TEXTURE_SIZE, maximum, 0);
            // ASTC LDR is core in ES 3.2, so a driver may omit the extension string there.
            boolean astc = has(extensions, "GL_KHR_texture_compression_astc_ldr")
                    || has(extensions, "GL_OES_texture_compression_astc")
                    || (version != null && version.startsWith("OpenGL ES 3.2"));
            boolean s3tc = has(extensions, "GL_EXT_texture_compression_s3tc");
            boolean rgtc = has(extensions, "GL_EXT_texture_compression_rgtc");
            boolean bptc = has(extensions, "GL_EXT_texture_compression_bptc");
            // ETC2 is core in GLES 3.0; some ES2 drivers expose it via extensions.
            boolean etc2 = (version != null && version.startsWith("OpenGL ES 3"))
                    || has(extensions, "GL_OES_compressed_ETC2_pixel_formats")
                    || has(extensions, "GL_EXT_texture_compression_etc2");
            return new Profile(astc, s3tc, s3tc, rgtc, rgtc, bptc, etc2,
                    maximum[0] > 0 ? maximum[0] : 8192);
        } finally {
            EGL14.eglMakeCurrent(display, EGL14.EGL_NO_SURFACE, EGL14.EGL_NO_SURFACE,
                    EGL14.EGL_NO_CONTEXT);
            if (surface != EGL14.EGL_NO_SURFACE) EGL14.eglDestroySurface(display, surface);
            if (context != EGL14.EGL_NO_CONTEXT) EGL14.eglDestroyContext(display, context);
            EGL14.eglTerminate(display);
            EGL14.eglReleaseThread();
        }
    }

    /** Probe failure stays conservative: no compressed format is assumed. */
    private static Profile unavailable() {
        return new Profile(false, false, false, false, false, false, false, 0);
    }

    private static String encode(Profile profile) {
        return (profile.astc ? 1 : 0) + "," + (profile.bc1 ? 1 : 0) + ","
                + (profile.bc3 ? 1 : 0) + "," + (profile.bc4 ? 1 : 0) + ","
                + (profile.bc5 ? 1 : 0) + "," + (profile.bc7 ? 1 : 0) + ","
                + (profile.etc2 ? 1 : 0) + "," + profile.maxTextureSize;
    }

    private static Profile parse(String value) {
        try {
            String[] parts = value.split(",");
            if (parts.length != 8) return null;
            return new Profile(parts[0].equals("1"), parts[1].equals("1"), parts[2].equals("1"),
                    parts[3].equals("1"), parts[4].equals("1"), parts[5].equals("1"),
                    parts[6].equals("1"), Integer.parseInt(parts[7]));
        } catch (RuntimeException malformed) {
            return null;
        }
    }

    private static boolean has(String extensions, String name) {
        return extensions != null
                && java.util.Arrays.asList(extensions.split(" ")).contains(name);
    }
}
