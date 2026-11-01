plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.arcad.app"
    compileSdk = 35
    defaultConfig {
        applicationId = "com.arcad.app"
        minSdk = 26
        targetSdk = 35
        versionCode = 25
        versionName = "0.6.1"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
    buildFeatures { compose = true }
    externalNativeBuild {
        cmake { path = file("src/main/cpp/CMakeLists.txt"); version = "3.22.1" }
    }
    buildTypes {
        getByName("debug") {
            externalNativeBuild {
                cmake { arguments += listOf("-DARCAD_ALLOW_NATIVE_STUB=ON") }
            }
        }
        getByName("release") {
            externalNativeBuild {
                cmake { arguments += listOf("-DARCAD_ALLOW_NATIVE_STUB=OFF") }
            }
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
}

// Release builds must compile and package the actual CADCraft Rust engine first.
// Debug builds may use the explicit JNI stub while native development is in progress.
val buildCadCraftAndroidNative by tasks.registering(Exec::class) {
    group = "build"
    description = "Builds the CADCraft Rust shared library for Android ABIs."
    workingDir(rootProject.projectDir)
    environment("ARCAD_REQUIRE_LOCKFILE", "1")
    commandLine("bash", "native/build-android.sh")
}

val verifyCadCraftReleaseNative by tasks.registering {
    group = "verification"
    description = "Fails release builds if the real CADCraft Rust libraries are absent."
    dependsOn(buildCadCraftAndroidNative)
    doLast {
        val nativeRoot = layout.projectDirectory.dir("src/main/jniLibs")
        listOf("arm64-v8a", "x86_64").forEach { abi ->
            val library = nativeRoot.file("$abi/libarcad_ffi.so").asFile
            check(library.isFile && library.length() > 0L) {
                "Missing real CADCraft Rust library for $abi: ${library.absolutePath}"
            }
        }
    }
}

tasks.matching { it.name == "preReleaseBuild" }.configureEach {
    dependsOn(verifyCadCraftReleaseNative)
}
// AGP's external-native tasks may be scheduled independently of merge tasks. Add
// the same prerequisite directly so CMake never races the Rust library build.
tasks.matching {
    it.name.contains("Release") &&
        (it.name.startsWith("configureCMake") || it.name.startsWith("buildCMake") ||
            it.name.startsWith("externalNativeBuild"))
}.configureEach {
    dependsOn(verifyCadCraftReleaseNative)
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.activity:activity-compose:1.10.0")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    debugImplementation("androidx.compose.ui:ui-tooling")
    testImplementation("junit:junit:4.13.2")
}
