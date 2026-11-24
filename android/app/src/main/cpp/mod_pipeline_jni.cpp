// ForTMR4A：统一导入入口的 JNI 暴露（Android 端调用 be_source_to_mod）。
//
// 设计：导入是低频操作，JNI 只做字符串/错误转换；真正的探测、转换、临时文件生命周期
// 都在 C 主链路 `native/pipeline` 里。Java 侧调用前必须保证 native 库已加载
// （由运行时/BemConverter 调用方保证）。
#include <jni.h>

#include <string>

#include "BetterEndfieldNext/SourcePipeline.h"

namespace {

std::string from_java(JNIEnv* env, jstring value) {
    if (!value) return {};
    const char* chars = env->GetStringUTFChars(value, nullptr);
    std::string text = chars ? chars : "";
    if (chars) env->ReleaseStringUTFChars(value, chars);
    return text;
}

}  // namespace

extern "C" {

JNIEXPORT jint JNICALL Java_dev_betterendfield_next_BemConverter_abiVersion(JNIEnv*, jclass) {
    return static_cast<jint>(be_pipeline_abi_version());
}

// 成功返回 null；失败返回错误文本（直接可展示/记日志）。
JNIEXPORT jstring JNICALL Java_dev_betterendfield_next_BemConverter_convert(
    JNIEnv* env, jclass, jstring source, jstring catalog_dir, jstring out_dir) {
    const std::string src = from_java(env, source);
    const std::string catalog = from_java(env, catalog_dir);
    const std::string out = from_java(env, out_dir);
    if (src.empty() || out.empty()) {
        return env->NewStringUTF("source/outDir must not be empty");
    }
    char err[512] = {0};
    const int rc = be_source_to_mod(src.c_str(),
                                    catalog.empty() ? nullptr : catalog.c_str(),
                                    out.c_str(), err, static_cast<unsigned>(sizeof err));
    if (rc == BE_PIPELINE_OK) return nullptr;
    if (err[0] == 0) {
        return env->NewStringUTF(rc == BE_PIPELINE_UNSUPPORTED
                                     ? "unsupported source"
                                     : "conversion failed");
    }
    return env->NewStringUTF(err);
}

}  // extern "C"
