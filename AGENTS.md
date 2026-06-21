# Rusty Sound

BPM (beats per minute) measurement app built with Rust and the Freya GUI library, targeting Android and desktop Linux.

## Project Structure

```
src/
  main.rs          # Desktop entry point
  lib.rs           # Android entry point (android_main via NativeActivity)
  app.rs           # Root UI component (mode switching, all UI layout)
  bpm/
    mod.rs
    tap.rs         # TapDetector: rolling average BPM from tap timestamps
    audio.rs       # Audio onset detection via cpal, global singleton pattern
AndroidApp/        # Gradle/Kotlin wrapper for Android APK
  app/
    build.gradle.kts  # Includes cargo-ndk build task with ANDROID_JAR setup
    src/main/
      AndroidManifest.xml
      java/dev/music/rusty_sound/MainActivity.kt
```

## Tech Stack

- **GUI**: [Freya](https://github.com/marc2332/freya) (0.4.x, git main branch) — Rust-native UI framework built on Skia
- **Audio**: [cpal](https://crates.io/crates/cpal) 0.17 — cross-platform audio I/O (ALSA on Linux, AAudio on Android)
- **Android Build**: cargo-ndk + Gradle + NativeActivity

## Building

### Desktop

```bash
cargo run --bin rusty_sound_desktop
```

System dependencies (Ubuntu/Debian):
```bash
sudo apt-get install libasound2-dev libgl1-mesa-dev libxkbcommon-dev libwayland-dev libfreetype-dev libfontconfig-dev libegl-dev libgles-dev libwayland-egl1
```

### Android

Requires Android SDK, NDK 27+, and cargo-ndk:
```bash
cargo install cargo-ndk
cd AndroidApp
./gradlew assembleDebug
```

The Gradle build task automatically invokes `cargo ndk` to cross-compile the Rust library for `arm64-v8a` and `x86_64` targets.

## Architecture Notes

- **Freya state**: Uses `use_state()` hooks. `State<T>` is `Copy` — read with `.read()`, write with `.write()`.
- **Audio detector**: Uses a global singleton (`LazyLock<Mutex<...>>`) because `cpal::Stream` can't easily be stored in Freya reactive state. The UI polls via a "Refresh BPM" button.
- **Conditional rendering**: Both tap and mic mode UI elements share the same element structure (same return type) with different text/callbacks to avoid type mismatch in if/else branches.
- **Android entry**: `lib.rs` uses `#[unsafe(no_mangle)] fn android_main(AndroidApp)` with `freya::android::AndroidPlugin` and `winit::platform::android::EventLoopBuilderExtAndroid`.

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):
```
feat: add new feature
fix: correct a bug
ci: update CI/build configuration
docs: documentation changes
refactor: code restructuring without behavior change
chore: maintenance tasks
```

## CI

GitHub Actions workflow (`.github/workflows/android.yml`) runs two jobs:
- **Desktop Build**: `cargo build --release --bin rusty_sound_desktop`, uploads binary artifact
- **Android APK**: Gradle + cargo-ndk, uploads debug APK artifact

Both use `if-no-files-found: error` on artifact upload to catch missing outputs.
