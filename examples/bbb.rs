use std::time::Duration;

use freya::prelude::*;
use freya_testing::{launch_test, FreyaEvent, MouseButton};
use tokio::time::sleep;

#[tokio::main]
async fn main(){
    fn stateful_app(cx: Scope) -> Element {
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

    let mut utils = launch_test(stateful_app);

    let rect = utils.root().get(0);
    let label = rect.get(0);

    utils.wait_for_update().await;

    loop {
        sleep(Duration::from_millis(1)).await;

        utils.push_event(FreyaEvent::Mouse {
            name: "click".to_string(),
            cursor: (5.0, 5.0).into(),
            button: Some(MouseButton::Left),
        });
    
        // Poll
        utils.wait_for_update().await;
    
        let text = label.get(0);
        println!("Counter: {:?}", text.text());
    }
}