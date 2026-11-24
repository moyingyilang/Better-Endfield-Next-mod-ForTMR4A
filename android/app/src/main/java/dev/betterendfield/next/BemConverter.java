package dev.betterendfield.next;

/**
 * 统一导入入口：BEM / EFMI / EFF3 / BEM ZIP → ForTMR2 mod 目录。
 *
 * <p>底层是 C 主链路 {@code be_source_to_mod}（native/pipeline）。调用前必须保证
 * native 库已由运行时加载；本类故意不在静态块里 loadLibrary，避免在模块进程里
 * 找不到注入库时崩溃。
 *
 * <p>EFMI/EFF 需要 {@code catalogDir}（BEN catalog 目录）；BEM 来源传 {@code null}。
 */
public final class BemConverter {
    private BemConverter() {}

    /** 成功返回 {@code null}；失败返回可展示的错误文本。 */
    public static native String convert(String source, String catalogDir, String outDir);

    /** 统一入口 ABI 版本。 */
    public static native int abiVersion();
}
