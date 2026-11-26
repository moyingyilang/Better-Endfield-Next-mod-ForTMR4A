package dev.betterendfield.next;

import android.content.Context;
import android.net.Uri;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayDeque;
import java.util.Deque;

/**
 * Archive / source fallback for the BEM import path.
 *
 * <p>The installer library carries the in-process Converter4BENeXt adapter plus
 * libarchive. Sources the normal BEM reader rejects are probed, extracted when
 * they are an archive (ZIP / 7-Zip / RAR4 / RAR5 / TAR and tar-family filters)
 * and then resolved to either a BEM file or an EFMI/EFF directory. EFMI/EFF
 * directories are converted to a temporary BEM file which is fed back into the
 * existing install pipeline.
 */
final class BemSource {
    private BemSource() {}

    static final int UNKNOWN = 0;
    static final int BEM = 1;
    static final int MOD_TOML = 2;
    static final int EFMI = 3;
    static final int EFF3 = 4;
    static final int ZIP = 5;

    private static final byte[] BEM_MAGIC = {'B', 'E', 'M', 0, 'P', 'K', 'G', 0};

    private static native int abiVersion();
    private static native int probe(String path);
    private static native String convertToBem(String source, String catalogDir, String outPath);
    private static native int probeArchive(String path);
    private static native String extractArchive(String source, String outDir);

    /** Raised when a source is recognized but conversion fails. */
    static final class Failure extends IOException {
        Failure(String message) { super(message); }
    }

    /**
     * Resolves a source URI to a BEM file, converting EFMI/EFF when needed.
     * Returns null when the source is not something this fallback understands.
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
        if (isBem(candidate)) return candidate;
        File out = new File(stage, "converted.bem");
        String error = convertToBem(candidate.getAbsolutePath(), ensureCatalog(app).getAbsolutePath(),
                out.getAbsolutePath());
        if (error != null) throw new Failure(error);
        return out.isFile() && out.length() > 0 ? out : null;
    }

    private static File locate(File raw, File stage) throws IOException {
        int kind = probe(raw.getAbsolutePath());
        if (kind == EFMI || kind == EFF3) return raw;
        if (probeArchive(raw.getAbsolutePath()) != 1) return null;
        File extracted = new File(stage, "extracted");
        String error = extractArchive(raw.getAbsolutePath(), extracted.getAbsolutePath());
        if (error != null) throw new Failure(error);
        return findSource(extracted, 0);
    }

    /** Depth-limited search for a BEM file or an EFMI/EFF directory. */
    private static File findSource(File directory, int depth) {
        if (directory == null || depth > 3) return null;
        int kind = probe(directory.getAbsolutePath());
        if (kind == EFMI || kind == EFF3) return directory;
        File[] children = directory.listFiles();
        if (children == null) return null;
        for (File child : children) {
            if (child.isFile() && isBem(child)) return child;
        }
        for (File child : children) {
            if (!child.isDirectory()) continue;
            File found = findSource(child, depth + 1);
            if (found != null) return found;
        }
        return null;
    }

    private static boolean isBem(File file) {
        if (!file.isFile()) return false;
        try (InputStream input = new FileInputStream(file)) {
            byte[] head = new byte[BEM_MAGIC.length];
            int total = 0;
            while (total < head.length) {
                int count = input.read(head, total, head.length - total);
                if (count < 0) return false;
                total += count;
            }
            for (int i = 0; i < BEM_MAGIC.length; ++i) {
                if (head[i] != BEM_MAGIC[i]) return false;
            }
            return true;
        } catch (IOException unreadable) {
            return false;
        }
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

    /** Extracts assets/catalog once; EFMI/EFF conversion needs it to match characters. */
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
