// ForTMR4A: source conversion JNI for the installer library.
//
// The app's import path already loads libbetterendfieldnext_installer. Expose
// the C++ adapter (Converter4BENeXt) there so EFMI / EFF packages can be turned
// into an ordinary BEM stream before the existing BEM install pipeline runs.
#include <jni.h>

#include <string>

#include "BetterEndfieldNext/SourceConvert.h"

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

JNIEXPORT jint JNICALL Java_dev_betterendfield_next_BemSource_abiVersion(JNIEnv*, jclass) {
    return static_cast<jint>(be_source_abi_version());
}

JNIEXPORT jint JNICALL Java_dev_betterendfield_next_BemSource_probe(JNIEnv* env, jclass, jstring path) {
    const std::string source = from_java(env, path);
    if (source.empty()) return -2;
    return be_source_probe(source.c_str());
}

JNIEXPORT jstring JNICALL Java_dev_betterendfield_next_BemSource_convertToBem(
    JNIEnv* env, jclass, jstring source, jstring catalog_dir, jstring out_path) {
    const std::string src = from_java(env, source);
    const std::string catalog = from_java(env, catalog_dir);
    const std::string out = from_java(env, out_path);
    if (src.empty() || out.empty()) return env->NewStringUTF("source/outPath must not be empty");
    char err[512] = {0};
    const int rc = be_source_convert_to_bem(src.c_str(),
                                            catalog.empty() ? nullptr : catalog.c_str(),
                                            out.c_str(), err, static_cast<unsigned>(sizeof err));
    if (rc == 0) return nullptr;
    if (err[0] == 0) {
        return env->NewStringUTF(rc == 3 ? "unsupported source" : "source conversion failed");
    }
    return env->NewStringUTF(err);
}

}  // extern "C"
