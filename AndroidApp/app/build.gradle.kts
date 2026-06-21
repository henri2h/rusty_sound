plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "dev.music.rusty_sound"
    compileSdk = 36

    defaultConfig {
        applicationId = "dev.music.rusty_sound"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        viewBinding = true
    }
}

val buildRustLibrary by tasks.registering(Exec::class) {
    workingDir = rootProject.projectDir.parentFile

    val androidHome = System.getenv("ANDROID_HOME")
        ?: System.getenv("ANDROID_SDK_ROOT")
        ?: run {
            val localProps = java.util.Properties()
            val localPropsFile = rootProject.file("local.properties")
            if (localPropsFile.exists()) localProps.load(localPropsFile.inputStream())
            localProps.getProperty("sdk.dir") ?: ""
        }

    environment("ANDROID_HOME", androidHome)
    environment("ANDROID_JAR", "$androidHome/platforms/android-36/android.jar")

    commandLine(
        "cargo", "ndk",
        "-o", "AndroidApp/app/src/main/jniLibs/",
        "-t", "arm64-v8a",
        "-t", "x86_64-linux-android",
        "--platform", "26",
        "build", "--lib", "--release"
    )
}

tasks.named("preBuild") {
    dependsOn(buildRustLibrary)
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.0")
}
