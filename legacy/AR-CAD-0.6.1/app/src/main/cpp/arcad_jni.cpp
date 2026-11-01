#include <jni.h>
#include <cstdint>
#include <limits>
#include <string>
#include <vector>
#include "arcad_ffi.h"

namespace {
std::string to_utf8(JNIEnv* env, jstring value) {
    if (value == nullptr) return {};
    const jchar* chars = env->GetStringChars(value, nullptr);
    if (chars == nullptr) return {};
    const jsize length = env->GetStringLength(value);
    std::string out;
    out.reserve(static_cast<size_t>(length) * 3);
    for (jsize i = 0; i < length; ++i) {
        uint32_t cp = chars[i];
        if (cp >= 0xD800 && cp <= 0xDBFF && i + 1 < length) {
            const uint32_t low = chars[i + 1];
            if (low >= 0xDC00 && low <= 0xDFFF) {
                cp = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                ++i;
            } else {
                cp = 0xFFFD;
            }
        } else if (cp >= 0xD800 && cp <= 0xDFFF) {
            cp = 0xFFFD;
        }
        if (cp <= 0x7F) out.push_back(static_cast<char>(cp));
        else if (cp <= 0x7FF) {
            out.push_back(static_cast<char>(0xC0 | (cp >> 6)));
            out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
        } else if (cp <= 0xFFFF) {
            out.push_back(static_cast<char>(0xE0 | (cp >> 12)));
            out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
            out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
        } else {
            out.push_back(static_cast<char>(0xF0 | (cp >> 18)));
            out.push_back(static_cast<char>(0x80 | ((cp >> 12) & 0x3F)));
            out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
            out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
        }
    }
    env->ReleaseStringChars(value, chars);
    return out;
}

jstring from_utf8(JNIEnv* env, const char* value) {
    if (value == nullptr) return env->NewString(nullptr, 0);
    const auto* bytes = reinterpret_cast<const unsigned char*>(value);
    std::vector<jchar> utf16;
    const size_t length = std::char_traits<char>::length(value);
    utf16.reserve(length);
    for (size_t i = 0; i < length;) {
        uint32_t cp = 0xFFFD;
        const unsigned char first = bytes[i++];
        if (first <= 0x7F) {
            cp = first;
        } else {
            size_t extra = 0;
            uint32_t minimum = 0;
            if ((first & 0xE0) == 0xC0) { cp = first & 0x1F; extra = 1; minimum = 0x80; }
            else if ((first & 0xF0) == 0xE0) { cp = first & 0x0F; extra = 2; minimum = 0x800; }
            else if ((first & 0xF8) == 0xF0) { cp = first & 0x07; extra = 3; minimum = 0x10000; }
            else { extra = 0; }
            bool valid = extra > 0 && i + extra <= length;
            for (size_t j = 0; valid && j < extra; ++j) {
                const unsigned char next = bytes[i + j];
                if ((next & 0xC0) != 0x80) valid = false;
                else cp = (cp << 6) | (next & 0x3F);
            }
            if (valid) {
                i += extra;
                if (cp < minimum || cp > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF)) cp = 0xFFFD;
            } else {
                cp = 0xFFFD;
            }
        }
        if (cp <= 0xFFFF) {
            utf16.push_back(static_cast<jchar>(cp));
        } else {
            cp -= 0x10000;
            utf16.push_back(static_cast<jchar>(0xD800 + (cp >> 10)));
            utf16.push_back(static_cast<jchar>(0xDC00 + (cp & 0x3FF)));
        }
    }
    return env->NewString(utf16.data(), static_cast<jsize>(utf16.size()));
}

void throw_message(JNIEnv* env, const char* message) {
    jclass errorClass = env->FindClass("java/lang/IllegalStateException");
    if (errorClass != nullptr) env->ThrowNew(errorClass, message);
}

void throw_native_error(JNIEnv* env, const char* fallback) {
    char* message = arcad_last_error();
    const char* text = (message != nullptr && message[0] != '\0') ? message : fallback;
    throw_message(env, text);
    if (message != nullptr) arcad_string_free(message);
}

jstring take_result(JNIEnv* env, char* result) {
    if (result == nullptr) { throw_native_error(env, "Native CAD command failed"); return nullptr; }
    jstring value = from_utf8(env, result);
    arcad_string_free(result);
    return value;
}

ArcadHandle* handle_from(jlong value) {
    return reinterpret_cast<ArcadHandle*>(static_cast<uintptr_t>(value));
}
jlong handle_to(ArcadHandle* value) {
    return static_cast<jlong>(reinterpret_cast<uintptr_t>(value));
}
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_arcad_app_NativeCadBridge_nativeIsAvailable(JNIEnv*, jobject) {
    return JNI_TRUE;
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeVersion(JNIEnv* env, jobject) {
    return from_utf8(env, arcad_core_version());
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_arcad_app_NativeCadBridge_nativeCreate(JNIEnv* env, jobject, jboolean metric) {
    ArcadHandle* handle = arcad_document_new(metric == JNI_TRUE ? 1 : 0);
    if (handle == nullptr) { throw_native_error(env, "Native CAD document creation failed"); return 0; }
    return handle_to(handle);
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_arcad_app_NativeCadBridge_nativeOpen(JNIEnv* env, jobject, jbyteArray bytes, jstring filename) {
    if (bytes == nullptr || filename == nullptr) { throw_message(env, "Missing CAD file input"); return 0; }
    const jsize length = env->GetArrayLength(bytes);
    if (length <= 0 || static_cast<size_t>(length) > 64U * 1024U * 1024U) { throw_message(env, "CAD file must be between 1 byte and 64 MB"); return 0; }
    std::string name = to_utf8(env, filename);
    if (name.empty() || name.size() > 4096 || name.find('\0') != std::string::npos) { throw_message(env, "Invalid CAD filename"); return 0; }
    jbyte* data = env->GetByteArrayElements(bytes, nullptr);
    if (data == nullptr) return 0;
    ArcadHandle* handle = arcad_document_open(reinterpret_cast<const uint8_t*>(data), static_cast<size_t>(length), name.c_str());
    env->ReleaseByteArrayElements(bytes, data, JNI_ABORT);
    if (handle == nullptr) { throw_native_error(env, "Could not open CAD drawing"); return 0; }
    return handle_to(handle);
}

extern "C" JNIEXPORT void JNICALL
Java_com_arcad_app_NativeCadBridge_nativeClose(JNIEnv*, jobject, jlong value) {
    if (value != 0) arcad_document_free(handle_from(value));
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_arcad_app_NativeCadBridge_nativeEntityCount(JNIEnv* env, jobject, jlong value) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return 0; }
    return static_cast<jlong>(arcad_document_entity_count(handle_from(value)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeReadNotes(JNIEnv* env, jobject, jlong value) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_read_notes(handle_from(value)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeExecute(JNIEnv* env, jobject, jlong value, jstring command, jstring params) {
    if (value == 0 || command == nullptr) { throw_message(env, "Invalid document or missing CAD command"); return nullptr; }
    std::string commandUtf8 = to_utf8(env, command);
    std::string paramsUtf8 = params == nullptr ? "{}" : to_utf8(env, params);
    if (commandUtf8.empty() || commandUtf8.size() > 256 || paramsUtf8.size() > 1024U * 1024U ||
        commandUtf8.find('\0') != std::string::npos || paramsUtf8.find('\0') != std::string::npos) {
        throw_message(env, "Invalid CAD command or parameters (empty, oversized, or contains NUL)");
        return nullptr;
    }
    return take_result(env, arcad_command_execute(handle_from(value), commandUtf8.c_str(), paramsUtf8.c_str()));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeStartCommand(JNIEnv* env, jobject, jlong value, jstring command) {
    if (value == 0 || command == nullptr) { throw_message(env, "Document is closed or command is missing"); return nullptr; }
    std::string commandUtf8 = to_utf8(env, command);
    if (commandUtf8.empty() || commandUtf8.size() > 256 || commandUtf8.find('\0') != std::string::npos) {
        throw_message(env, "Invalid CAD command name");
        return nullptr;
    }
    return take_result(env, arcad_command_start(handle_from(value), commandUtf8.c_str()));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeSetViewport(JNIEnv* env, jobject, jlong value, jdouble width, jdouble height) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_set_viewport(handle_from(value), width, height));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativePointerMove(JNIEnv* env, jobject, jlong value, jdouble x, jdouble y, jboolean shift, jdouble now) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_pointer_move(handle_from(value), x, y, shift == JNI_TRUE ? 1 : 0, now));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativePointerClick(JNIEnv* env, jobject, jlong value, jdouble x, jdouble y, jboolean shift, jdouble now) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_pointer_click(handle_from(value), x, y, shift == JNI_TRUE ? 1 : 0, now));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeInputCommand(JNIEnv* env, jobject, jlong value, jstring inputJson) {
    if (value == 0 || inputJson == nullptr) { throw_message(env, "Document is closed or interactive input is missing"); return nullptr; }
    std::string inputUtf8 = to_utf8(env, inputJson);
    if (inputUtf8.empty() || inputUtf8.size() > 1024U * 1024U || inputUtf8.find('\0') != std::string::npos) {
        throw_message(env, "Invalid interactive input JSON");
        return nullptr;
    }
    return take_result(env, arcad_command_input(handle_from(value), inputUtf8.c_str()));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeUndo(JNIEnv* env, jobject, jlong value) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_undo(handle_from(value)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_arcad_app_NativeCadBridge_nativeRedo(JNIEnv* env, jobject, jlong value) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    return take_result(env, arcad_document_redo(handle_from(value)));
}

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_arcad_app_NativeCadBridge_nativeSave(JNIEnv* env, jobject, jlong value, jstring filename) {
    if (value == 0 || filename == nullptr) { throw_message(env, "Document is closed or filename is missing"); return nullptr; }
    std::string name = to_utf8(env, filename);
    if (name.empty() || name.size() > 4096 || name.find('\0') != std::string::npos) { throw_message(env, "Invalid CAD filename"); return nullptr; }
    ArcadBuffer* buffer = arcad_document_save(handle_from(value), name.c_str());
    if (buffer == nullptr) { throw_native_error(env, "Could not save CAD drawing"); return nullptr; }
    const size_t length = arcad_buffer_len(buffer);
    if (length > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        arcad_buffer_free(buffer);
        return nullptr;
    }
    jbyteArray output = env->NewByteArray(static_cast<jsize>(length));
    if (output != nullptr && length > 0) {
        const uint8_t* data = arcad_buffer_data(buffer);
        env->SetByteArrayRegion(output, 0, static_cast<jsize>(length), reinterpret_cast<const jbyte*>(data));
    }
    arcad_buffer_free(buffer);
    return output;
}

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_arcad_app_NativeCadBridge_nativeRenderPng(JNIEnv* env, jobject, jlong value, jint width, jint height) {
    if (value == 0) { throw_message(env, "CAD document is closed"); return nullptr; }
    if (width < 16 || height < 16 || width > 4096 || height > 4096 ||
        static_cast<int64_t>(width) * static_cast<int64_t>(height) > 16777216LL) {
        throw_message(env, "Preview dimensions must be 16..4096 pixels and at most 16 megapixels");
        return nullptr;
    }
    ArcadBuffer* buffer = arcad_document_render_png(handle_from(value), static_cast<uint32_t>(width), static_cast<uint32_t>(height));
    if (buffer == nullptr) { throw_native_error(env, "Could not render CAD preview"); return nullptr; }
    const size_t length = arcad_buffer_len(buffer);
    if (length > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        arcad_buffer_free(buffer);
        throw_message(env, "Rendered preview exceeds the Android byte-array limit");
        return nullptr;
    }
    jbyteArray output = env->NewByteArray(static_cast<jsize>(length));
    if (output != nullptr && length > 0) {
        const uint8_t* data = arcad_buffer_data(buffer);
        env->SetByteArrayRegion(output, 0, static_cast<jsize>(length), reinterpret_cast<const jbyte*>(data));
    }
    arcad_buffer_free(buffer);
    return output;
}
