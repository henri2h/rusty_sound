pub mod app;
pub mod bpm;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(droid_app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let event_loop = winit::event_loop::EventLoop::<freya::NativeEvent>::with_user_event()
        .with_android_app(droid_app.clone())
        .build()
        .expect("Failed to build event loop");

    freya::launch(
        freya::LaunchConfig::new()
            .with_plugin(freya_android::AndroidPlugin::new(droid_app))
            .with_window(freya::WindowConfig::new(app::app))
            .with_event_loop(event_loop),
    );
}
