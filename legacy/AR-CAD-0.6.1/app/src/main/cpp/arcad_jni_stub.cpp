#include <jni.h>

// Development-only fallback: lets the Android UI build before Rust/NDK libraries exist.
// NativeCadBridge.ensureLoaded() checks this flag and refuses all CAD engine operations.
extern "C" JNIEXPORT jboolean JNICALL
Java_com_arcad_app_NativeCadBridge_nativeIsAvailable(JNIEnv*, jobject) {
    return JNI_FALSE;
}

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_arcad_app_NativeCadBridge_nativeRenderPng(JNIEnv* env, jobject, jlong, jint, jint) {
    jclass errorClass = env->FindClass("java/lang/UnsupportedOperationException");
    if (errorClass != nullptr) env->ThrowNew(errorClass, "CADCraft native renderer is unavailable in the development stub");
    return nullptr;
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeSetViewport(JNIEnv* env, jobject, jlong, jdouble, jdouble) {
    jclass c = env->FindClass("java/lang/UnsupportedOperationException");
    if (c != nullptr) env->ThrowNew(c, "CADCraft native viewport is unavailable in the development stub");
    return nullptr;
}
extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativePointerMove(JNIEnv* env, jobject, jlong, jdouble, jdouble, jboolean, jdouble) {
    jclass c = env->FindClass("java/lang/UnsupportedOperationException");
    if (c != nullptr) env->ThrowNew(c, "CADCraft native pointer input is unavailable in the development stub");
    return nullptr;
}
extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativePointerClick(JNIEnv* env, jobject, jlong, jdouble, jdouble, jboolean, jdouble) {
    jclass c = env->FindClass("java/lang/UnsupportedOperationException");
    if (c != nullptr) env->ThrowNew(c, "CADCraft native pointer input is unavailable in the development stub");
    return nullptr;
}
