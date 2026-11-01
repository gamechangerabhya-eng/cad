# JNI exports use the Java class and native method names as their symbol contract.
# Keep the bridge intact in minified release builds.
-keep class com.arcad.app.NativeCadBridge { *; }
