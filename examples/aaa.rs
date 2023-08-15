#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::time::Duration;

use freya::prelude::*;
use tokio::time::sleep;

fn main() {
    launch(app);
}

fn app(cx: Scope) -> Element {
    let counter = use_state(cx, || 1);
    if *counter.get() > 50 {
        counter.set(1);
    }

    cx.use_hook(|| {
        to_owned![counter];
        cx.push_future(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                counter += 1;
            }
        })
    });

    render!(
        rect {
            overflow: "clip",
            width: "100%",
            height: "100%",
            background: "red",
            direction: "both",
            label {
                "{counter}"
            }
            rect {
                for el in 0..*counter.get() {
                    rsx!(
                        rect {
                            key: "{el}",
                            label {
                                "{el}"
                            }
                        }
                    )
                }
            }
            rect {
                for el in 0..*counter.get() {
                    rsx!(
                        rect {
                            key: "{el}",
                            label {
                                "{el}"
                            }
                        }
                    )
                }
            }
        }
    )
}
