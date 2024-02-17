//! # Testing
//!
//! `freya-testing` is a special renderer that let's you run your components in a headless environment.
//! This will let you easily write unit tests for your components.
//!
//! ## Getting started
//!
//! Add `freya-testing`:
//!
//! ```toml
//! [dev-dependencies]
//! freya-testing = "0.1"
//! ```
//!
//! You can use the `launch_test` function to run the tests of your component, it will return you a set of utilities for you to interact with the component.
//!
//! For example, this will launch a state-less component and assert that it renders a label with the text `"Hello World!"`.
//!
//! ```rust, no_run
//! #[tokio::test]
//! async fn test() {
//!     fn our_component() -> Element {
//!         rsx!(
//!             label {
//!                 "Hello World!"
//!             }
//!         )
//!     }
//!
//!     let mut utils = launch_test(our_component);
//!
//!     let root = utils.root();
//!     let label = root.get(0);
//!     let label_text = label.get(0);
//!
//!     assert_eq!(label_text.text(), Some("Hello World!"));
//! }
//! ```
//!
//! The `root()` function will give you the Root node of your app, then, with the `get` function you can retrieve a Node from it's parent given it's index position.
//!
//! ## Dynamic components
//!
//! If the component has logic that might execute asynchronously, you will need to wait for the component to update using the `wait_for_update` function before asserting the result.
//!
//! Here, the component has a state that is `false` by default, but, once mounted it will update the state to `true`.
//!
//! ```rust, no_run
//! #[tokio::test]
//! async fn dynamic_test() {
//!     fn dynamic_component() -> Element {
//!         let mut state = use_signal(|| false);
//!
//!         use_hook(move || {
//!             state.set(true);
//!         });
//!
//!         rsx!(
//!             label {
//!                 "Is enabled? {state}"
//!             }
//!         )
//!     }
//!
//!     let mut utils = launch_test(dynamic_component);
//!
//!     let root = utils.root();
//!     let label = root.get(0);
//!
//!     assert_eq!(label.get(0).text(), Some("Is enabled? false"));
//!
//!     // This will run the `use_effect` and update the state.
//!     utils.wait_for_update().await;
//!
//!     assert_eq!(label.get(0).text(), Some("Is enabled? true"));
//! }
//! ```
//!
//! ## Events
//!
//! We can also simulate events on the component, for example, we can simulate a click event on a `rect` and assert that the state has been updated.
//!
//! ```rust, no_run
//! #[tokio::test]
//! async fn event_test() {
//!     fn event_component() -> Element {
//!         let mut enabled = use_signal(|| false);
//!
//!         rsx!(
//!             rect {
//!                 width: "100%",
//!                 height: "100%",
//!                 background: "red",
//!                 onclick: move |_| {
//!                     enabled.set(true);
//!                 },
//!                 label {
//!                     "Is enabled? {enabled}"
//!                 }
//!             }
//!         )
//!     }
//!
//!     let mut utils = launch_test(event_component);
//!
//!     let rect = utils.root().get(0);
//!     let label = rect.get(0);
//!
//!     utils.wait_for_update().await;
//!
//!     let text = label.get(0);
//!     assert_eq!(text.text(), Some("Is enabled? false"));
//!
//!     // Push a click event to the events queue
//!     utils.push_event(FreyaEvent::Mouse {
//!         name: "click",
//!         cursor: (5.0, 5.0).into(),
//!         button: Some(MouseButton::Left),
//!     });
//!
//!     // Run the queued events and update the state
//!     utils.wait_for_update().await;
//!
//!     // Because the click event was executed and the state updated, now the text has changed too!
//!     let text = label.get(0);
//!     assert_eq!(text.text(), Some("Is enabled? true"));
//! }
//! ```
//!
//! ## Testing configuration
//!
//! The `launch_test` comes with a default configuration, but you can also pass your own config with the `launch_test_with_config` function.
//!
//! Here is an example of how to can set our custom window size:
//!
//! ```rust, no_run
//! #[tokio::test]
//! async fn test() {
//!     fn our_component() -> Element {
//!         rsx!(
//!             label {
//!                 "Hello World!"
//!             }
//!         )
//!     }
//!
//!     let mut utils = launch_test_with_config(
//!         our_component,
//!         TestingConfig {
//!             size: (500.0, 800.0).into(),
//!             ..TestingConfig::default()
//!         },
//!     );
//!
//!     let root = utils.root();
//!     let label = root.get(0);
//!     let label_text = label.get(0);
//!
//!     assert_eq!(label_text.text(), Some("Hello World!"));
//! }
//! ````
