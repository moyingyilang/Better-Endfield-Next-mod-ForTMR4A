package dev.betterendfield.next;

import android.content.Context;
import android.net.Uri;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

/**
 * EFMI / EFF source fallback for the BEM import path.
 *
 * <p>The installer library carries the in-process Converter4BENeXt adapter. A
 * source that the normal BEM reader rejects is first probed (and unpacked when
 * it is a ZIP); EFMI/EFF directories are converted to a temporary BEM file which
 * is then fed back into the existing install pipeline.
 */
final class BemSource {
    private BemSource() {}

    static final int UNKNOWN = 0;
    static final int BEM = 1;
    static final int MOD_TOML = 2;
    static final int EFMI = 3;
    static final int EFF3 = 4;
    static final int ZIP = 5;

    private static native int abiVersion();
    private static native int probe(String path);
    private static native String convertToBem(String source, String catalogDir, String outPath);

    /** Last probe of the native library; used to keep a clear error for unsupported sources. */
    static final class Failure extends IOException {
        Failure(String message) { super(message); }
    }

    /**
     * Converts a source URI to a temporary BEM file, or returns null when the
     * source is not EFMI/EFF (the caller keeps its original error).
     */
    static File prepare(Context context, Uri uri, File stage) throws IOException {
        Context app = context.getApplicationContext();
        BemInstaller.loadCodec();
        File raw = new File(stage, "source.bin");
        try (InputStream input = app.getContentResolver().openInputStream(uri);
             OutputStream output = new FileOutputStream(raw)) {
            if (input == null) throw new IOException(Text.of("无法打开包"));
            copy(input, output, 8L * 1024 * 1024 * 1024);
        }
        File candidate = locate(raw, stage);
        if (candidate == null) return null;
        File out = new File(stage, "converted.bem");
        String error = convertToBem(candidate.getAbsolutePath(), ensureCatalog(app).getAbsolutePath(),
                out.getAbsolutePath());
        if (error != null) throw new Failure(error);
        return out.isFile() && out.length() > 0 ? out : null;
    }

    private static File locate(File raw, File stage) throws IOException {
        int kind = probe(raw.getAbsolutePath());
        if (kind == EFMI || kind == EFF3) return raw;
        if (kind != ZIP) return null;
        File unpacked = new File(stage, "source");
        if (!unpacked.mkdir() && !unpacked.isDirectory()) throw new IOException(Text.of("无法创建解包目录"));
        unzip(raw, unpacked);
        int rootKind = probe(unpacked.getAbsolutePath());
        if (rootKind == EFMI || rootKind == EFF3) return unpacked;
        File[] children = unpacked.listFiles();
        if (children != null) {
            for (File child : children) {
                if (!child.isDirectory()) continue;
                int childKind = probe(child.getAbsolutePath());
                if (childKind == EFMI || childKind == EFF3) return child;
            }
        }
        return null;
    }

    private static void copy(InputStream input, OutputStream output, long limit) throws IOException {
        byte[] buffer = new byte[65536];
        long total = 0;
        int count;
        while ((count = input.read(buffer)) != -1) {
            total += count;
            if (total > limit) throw new IOException(Text.of("包体过大"));
            output.write(buffer, 0, count);
        }
    }

    private static void unzip(File archive, File root) throws IOException {
        try (ZipInputStream zip = new ZipInputStream(new FileInputStream(archive))) {
            ZipEntry entry;
            int entries = 0;
            long total = 0;
            byte[] buffer = new byte[65536];
            while ((entry = zip.getNextEntry()) != null) {
                if (++entries > 4096) throw new IOException(Text.of("ZIP 条目过多"));
                String name = entry.getName().replace('\\', '/');
                if (name.startsWith("/") || name.contains("../") || name.equals("..")) {
                    throw new IOException(Text.of("ZIP 含非法路径"));
                }
                File target = new File(root, name);
                if (!target.getCanonicalPath().startsWith(root.getCanonicalPath() + File.separator)
                        && !target.getCanonicalPath().equals(root.getCanonicalPath())) {
                    throw new IOException(Text.of("ZIP 含非法路径"));
                }
                if (entry.isDirectory()) {
                    if (!target.isDirectory() && !target.mkdirs()) throw new IOException(Text.of("无法创建解包目录"));
                    continue;
                }
                File parent = target.getParentFile();
                if (parent != null && !parent.isDirectory() && !parent.mkdirs()) {
                    throw new IOException(Text.of("无法创建解包目录"));
                }
                try (OutputStream output = new FileOutputStream(target)) {
                    int count;
                    while ((count = zip.read(buffer)) != -1) {
                        total += count;
                        if (total > 8L * 1024 * 1024 * 1024) throw new IOException(Text.of("包体过大"));
                        output.write(buffer, 0, count);
                    }
                }
            }
        }
    }

    /** Extracts assets/catalog once; the converter needs it to match characters. */
    private static File ensureCatalog(Context app) throws IOException {
        File root = new File(app.getFilesDir(), "converter-catalog");
        File marker = new File(root, ".complete");
        if (marker.isFile()) return root;
        if (root.isDirectory()) deleteTree(root);
        if (!root.mkdirs() && !root.isDirectory()) throw new IOException(Text.of("无法创建 catalog 目录"));
        String[] names;
        try {
            names = app.getAssets().list("catalog");
        } catch (IOException missing) {
            names = null;
        }
        if (names == null || names.length == 0) {
            // No bundled catalog: let the converter resolve whatever it can.
            if (!marker.createNewFile()) throw new IOException(Text.of("无法写入 catalog 标记"));
            return root;
        }
        for (String name : names) {
            if (!name.endsWith(".json")) continue;
            File target = new File(root, name);
            try (InputStream input = app.getAssets().open("catalog/" + name);
                 OutputStream output = new FileOutputStream(target)) {
                copy(input, output, 64L * 1024 * 1024);
            }
        }
        if (!marker.createNewFile()) throw new IOException(Text.of("无法写入 catalog 标记"));
        return root;
    }

    private static void deleteTree(File file) {
        Deque<File> pending = new ArrayDeque<>();
        pending.push(file);
        while (!pending.isEmpty()) {
            File current = pending.pop();
            File[] children = current.listFiles();
            if (children != null) {
                for (File child : children) {
                    if (child.isDirectory()) pending.push(child);
                    else child.delete();
                }
            }
            current.delete();
        }
    }
}
