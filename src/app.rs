use freya::prelude::*;

use crate::bpm::audio;
use crate::bpm::tap::TapDetector;

pub fn app() -> impl IntoElement {
    let mut mode = use_state(|| true); // true = tap, false = mic
    let mut tap_detector = use_state(|| TapDetector::new(8, 2.0));
    let mut tap_bpm = use_state(|| None::<f64>);
    let mut tap_count_state = use_state(|| 0usize);
    let mut mic_bpm = use_state(|| None::<f64>);
    let mut mic_listening = use_state(|| false);
    let mut mic_status = use_state(|| String::from("Tap Start to begin"));

    let is_tap = *mode.read();
    let listening = *mic_listening.read();

    let displayed_bpm = if is_tap {
        *tap_bpm.read()
    } else {
        *mic_bpm.read()
    };
    let bpm_text = match displayed_bpm {
        Some(v) => format!("{:.1}", v),
        None => "---".to_string(),
    };

    let tap_count = *tap_count_state.read();
    let status_text = if is_tap {
        format!("Taps: {}/8", tap_count)
    } else if listening {
        "Listening...".to_string()
    } else {
        mic_status.read().clone()
    };

    let main_btn_text = if is_tap {
        "TAP"
    } else if listening {
        "Stop"
    } else {
        "Start Listening"
    };

    let secondary_btn_text = if is_tap { "Reset" } else { "Refresh BPM" };

    rect()
        .expanded()
        .background((26, 26, 46))
        .color((255, 255, 255))
        .padding(Gaps::new_all(16.0))
        .child(
            rect()
                .width(Size::fill())
                .center()
                .padding(Gaps::new_all(16.0))
                .child(label().text("Rusty Sound").font_size(32.0)),
        )
        .child(
            rect()
                .width(Size::fill())
                .center()
                .padding(Gaps::new_all(24.0))
                .child(
                    label()
                        .text(bpm_text.clone())
                        .font_size(80.0)
                        .color((233, 69, 96)),
                )
                .child(
                    label()
                        .text("BPM")
                        .font_size(20.0)
                        .color((160, 160, 176)),
                ),
        )
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .center()
                .spacing(16.0)
                .padding(Gaps::new_all(8.0))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if !is_tap && listening {
                                audio::stop_listening();
                                mic_listening.set(false);
                                mic_status.set("Stopped".to_string());
                            }
                            mode.set(true);
                        })
                        .child(if is_tap { "> Tap" } else { "Tap" }),
                )
                .child(
                    Button::new()
                        .on_press(move |_| mode.set(false))
                        .child(if !is_tap { "> Mic" } else { "Mic" }),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .center()
                .padding(Gaps::new_all(8.0))
                .child(
                    label()
                        .text(status_text.clone())
                        .font_size(16.0)
                        .color((160, 160, 176)),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::percent(35.0))
                .center()
                .padding(Gaps::new_all(16.0))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if is_tap {
                                let bpm = tap_detector.write().tap();
                                tap_bpm.set(bpm);
                                tap_count_state.set(tap_detector.read().tap_count());
                            } else if listening {
                                audio::stop_listening();
                                mic_listening.set(false);
                                mic_status.set("Stopped".to_string());
                            } else {
                                match audio::start_listening() {
                                    Ok(()) => {
                                        mic_listening.set(true);
                                    }
                                    Err(e) => {
                                        mic_status.set(e);
                                    }
                                }
                            }
                        })
                        .child(main_btn_text),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .center()
                .padding(Gaps::new_all(8.0))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if is_tap {
                                tap_detector.write().reset();
                                tap_bpm.set(None);
                                tap_count_state.set(0);
                            } else {
                                mic_bpm.set(audio::current_mic_bpm());
                            }
                        })
                        .child(secondary_btn_text),
                ),
        )
}
