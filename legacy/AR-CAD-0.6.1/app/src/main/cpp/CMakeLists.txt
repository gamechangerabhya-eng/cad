cmake_minimum_required(VERSION 3.22.1)
project(arcad_jni LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(ARCAD_ALLOW_NATIVE_STUB "Allow the UI-only JNI stub for debug builds" OFF)
set(ARCAD_RUST_LIB "${CMAKE_CURRENT_LIST_DIR}/../jniLibs/${ANDROID_ABI}/libarcad_ffi.so")
if(EXISTS "${ARCAD_RUST_LIB}")
    add_library(arcad_ffi SHARED IMPORTED)
    set_target_properties(arcad_ffi PROPERTIES IMPORTED_LOCATION "${ARCAD_RUST_LIB}")
    add_library(arcad_jni SHARED arcad_jni.cpp)
    target_include_directories(arcad_jni PRIVATE "${CMAKE_CURRENT_LIST_DIR}/../../../../native/include")
    target_link_libraries(arcad_jni PRIVATE arcad_ffi log)
    message(STATUS "AR CAD: CADCraft native engine enabled for ${ANDROID_ABI}")
elseif(ARCAD_ALLOW_NATIVE_STUB)
    # Debug-only convenience. NativeCadBridge.ensureLoaded() reports unavailable;
    # this path must never be used by release variants.
    add_library(arcad_jni SHARED arcad_jni_stub.cpp)
    message(WARNING "AR CAD: debug-only JNI stub selected for ${ANDROID_ABI}; CAD operations are disabled.")
else()
    message(FATAL_ERROR
        "AR CAD release requires the real CADCraft Rust library: ${ARCAD_RUST_LIB}. "
        "Run the Gradle release task, which builds Rust through native/build-android.sh. "
        "The JNI stub is permitted only when ARCAD_ALLOW_NATIVE_STUB=ON for debug builds.")
endif()
