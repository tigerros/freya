use dioxus::prelude::*;
use dioxus_core::{ElementId, Scope};
use dioxus_elements::MouseButton;
use dioxus_native_core::tree::TreeView;
use dioxus_native_core::NodeId;
use dioxus_native_core::{node::NodeType, real_dom::RealDom};
use dioxus_router::*;
use freya_components::*;
use freya_core::events::{DomEvent, DomEventData};
use freya_elements as dioxus_elements;
use freya_elements::events_data::{Length, MouseData, Point2D};
use freya_elements::MouseEvent;
use freya_hooks::{use_init_focus, use_theme};
use freya_node_state::{AttributeType, CustomAttributeValues, NodeState, ShadowSettings};
use freya_renderer::HoveredNode;
use rustc_hash::FxHashSet;
use skia_safe::Color;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;

/// Launch a Component with the devtools panel enabled.
pub fn with_devtools(
    rdom: Arc<Mutex<RealDom<NodeState, CustomAttributeValues>>>,
    root: fn(cx: Scope) -> Element,
    mutations_receiver: UnboundedReceiver<()>,
    hovered_node: HoveredNode,
    event_emitter: UnboundedSender<DomEvent>,
) -> VirtualDom {
    fn app(cx: Scope<DomProps>) -> Element {
        use_init_focus(cx);

        #[allow(non_snake_case)]
        let Root = cx.props.root;
        let mutations_receiver = cx.props.mutations_receiver.clone();
        let hovered_node = cx.props.hovered_node.clone();
        let event_emitter = cx.props.event_emitter.clone();

        render!(
            rect {
                width: "100%",
                height: "100%",
                direction: "horizontal",
                container {
                    height: "100%",
                    width: "calc(100% - 350)",
                    Root { },
                }
                rect {
                    background: "rgb(40, 40, 40)",
                    height: "100%",
                    width: "350",
                    ThemeProvider {
                        DevTools {
                            rdom: cx.props.rdom.clone(),
                            mutations_receiver: mutations_receiver
                            hovered_node: hovered_node
                            event_emitter: event_emitter
                        }
                    }
                }
            }
        )
    }

    struct DomProps {
        root: fn(cx: Scope) -> Element,
        rdom: Arc<Mutex<RealDom<NodeState, CustomAttributeValues>>>,
        mutations_receiver: Arc<Mutex<UnboundedReceiver<()>>>,
        hovered_node: HoveredNode,
        event_emitter: UnboundedSender<DomEvent>,
    }

    let mutations_receiver = Arc::new(Mutex::new(mutations_receiver));

    VirtualDom::new_with_props(
        app,
        DomProps {
            root,
            rdom,
            mutations_receiver,
            hovered_node,
            event_emitter,
        },
    )
}

#[derive(Clone)]
struct TreeNode {
    tag: String,
    id: NodeId,
    element_id: ElementId,
    height: u16,
    #[allow(dead_code)]
    text: Option<String>,
    state: NodeState,
    listeners: Option<FxHashSet<String>>,
}

#[derive(Props)]
pub struct DevToolsProps {
    rdom: Arc<Mutex<RealDom<NodeState, CustomAttributeValues>>>,
    mutations_receiver: Arc<Mutex<UnboundedReceiver<()>>>,
    hovered_node: HoveredNode,
    event_emitter: UnboundedSender<DomEvent>,
}

impl PartialEq for DevToolsProps {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[allow(non_snake_case)]
pub fn DevTools(cx: Scope<DevToolsProps>) -> Element {
    let children = use_state(cx, Vec::<TreeNode>::new);
    let theme = use_theme(cx);
    let theme = theme.read();

    use_effect(cx, (), move |_| {
        let rdom = cx.props.rdom.clone();
        let mutations_receiver = cx.props.mutations_receiver.clone();
        let children = children.clone();
        async move {
            let mut mutations_receiver = mutations_receiver.lock().unwrap();
            loop {
                if mutations_receiver.recv().await.is_some() {
                    sleep(Duration::from_millis(10)).await;

                    let rdom = rdom.lock().unwrap();
                    let mut new_children = Vec::new();

                    let mut root_found = false;
                    let mut devtools_found = false;

                    rdom.traverse_depth_first(|node| {
                        let height = rdom.tree.height(node.node_data.node_id).unwrap();
                        if height == 2 {
                            if !root_found {
                                root_found = true;
                            } else {
                                devtools_found = true;
                            }
                        }

                        if !devtools_found {
                            let (tag, text, listeners) = match &node.node_data.node_type {
                                NodeType::Text { text, .. } => {
                                    ("text".to_string(), Some(text.clone()), None)
                                }
                                NodeType::Element { tag, listeners, .. } => {
                                    (tag.clone(), None, Some(listeners.clone()))
                                }
                                NodeType::Placeholder => ("placeholder".to_string(), None, None),
                            };

                            if let Some(element_id) = node.node_data.element_id {
                                new_children.push(TreeNode {
                                    height,
                                    id: node.node_data.node_id,
                                    tag,
                                    text,
                                    listeners,
                                    element_id,
                                    state: node.state.clone(),
                                });
                            }
                        }
                    });
                    children.set(new_children);
                }
            }
        }
    });

    let selected_node_id = use_state::<Option<NodeId>>(&cx, || None);

    let selected_node = children.iter().find(|c| {
        if let Some(n_id) = selected_node_id.get() {
            n_id == &c.id
        } else {
            false
        }
    });

    render!(
        rect {
            width: "100%",
            height: "100%",
            color: theme.body.color,
            Router {
                initial_url: "freya://freya/elements".to_string(),
                TabsBar {
                    TabButton {
                        to: "/elements",
                        label: "Elements"
                    }
                }
                Route {
                    to: "/elements",
                    NodesTree {
                        nodes: children,
                        height: "calc(100% - 35)",
                        selected_node_id: &None,
                        onselected: |node: &TreeNode| {
                            if let Some(hovered_node) = &cx.props.hovered_node {
                                hovered_node.lock().unwrap().replace(node.id);
                            }
                            selected_node_id.set(Some(node.id));
                        }
                    }
                }
                Route {
                    to: "/elements/style",
                    NodesTree {
                        nodes: children,
                        height: "calc(50% - 35)",
                        selected_node_id: selected_node_id.get(),
                        onselected: |node: &TreeNode| {
                            if let Some(hovered_node) = &cx.props.hovered_node {
                                hovered_node.lock().unwrap().replace(node.id);
                            }
                            selected_node_id.set(Some(node.id));
                        }
                    }
                    selected_node.and_then(|selected_node| {
                        Some(rsx!(
                            NodeInspectorStyle {
                                node: selected_node
                            }
                        ))
                    })
                }
                Route {
                    to: "/elements/listeners",
                    NodesTree {
                        nodes: children,
                        height: "calc(50% - 35)",
                        selected_node_id: selected_node_id.get(),
                        onselected: |node: &TreeNode| {
                            if let Some(hovered_node) = &cx.props.hovered_node {
                                hovered_node.lock().unwrap().replace(node.id);
                            }
                            selected_node_id.set(Some(node.id));
                        }
                    }
                    selected_node.and_then(|selected_node| {
                        Some(rsx!(
                            NodeInspectorListeners {
                                event_emitter: cx.props.event_emitter.clone(),
                                node: selected_node
                            }
                        ))
                    })
                }
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn NodesTree<'a>(
    cx: Scope<'a>,
    nodes: &'a Vec<TreeNode>,
    height: &'a str,
    selected_node_id: &'a Option<NodeId>,
    onselected: EventHandler<'a, &'a TreeNode>,
) -> Element<'a> {
    let router = use_router(&cx);

    render!(VirtualScrollView {
        width: "100%",
        height: "{height}",
        padding: "30",
        show_scrollbar: true,
        length: nodes.len() as i32,
        item_size: 27.0,
        builder_values: (nodes, selected_node_id, onselected, router),
        builder: Box::new(move |(_k, i, values)| {
            let (nodes, selected_node_id, onselected, router) = values.unwrap();
            let node = nodes.get(i as usize).unwrap();
            rsx! {
                NodeElement {
                    key: "{node.id:?}",
                    is_selected: Some(node.id) == **selected_node_id,
                    onselected: |node: &TreeNode| {
                        onselected.call(node);
                        router.push_route("/elements/style", None, None)
                    }
                    node: node
                }
            }
        })
    })
}

#[derive(Props)]
struct TabsBarProps<'a> {
    pub children: Element<'a>,
}

#[allow(non_snake_case)]
fn TabsBar<'a>(cx: Scope<'a, TabsBarProps<'a>>) -> Element<'a> {
    let theme = use_theme(&cx);
    let button_theme = &theme.read().button;
    render!(
        container {
            background: "{button_theme.background}",
            direction: "horizontal",
            height: "35",
            width: "100%",
            color: "{button_theme.font_theme.color}",
            &cx.props.children
        }
    )
}

#[derive(Props)]
struct TabButtonProps<'a> {
    pub to: &'a str,
    pub label: &'a str,
}

#[allow(non_snake_case)]
fn TabButton<'a>(cx: Scope<'a, TabButtonProps<'a>>) -> Element<'a> {
    let theme = use_theme(&cx);
    let button_theme = &theme.read().button;

    let background = use_state(cx, || <&str>::clone(&button_theme.background));
    let set_background = background.setter();

    use_effect(cx, &button_theme.clone(), move |button_theme| async move {
        set_background(button_theme.background);
    });

    let content = cx.props.label;
    render!(
        container {
            background: "{background}",
            onmouseover: move |_| {
                    background.set(theme.read().button.hover_background);
            },
            onmouseleave: move |_| {
                background.set(theme.read().button.background);
            },
            width: "125",
            radius: "7",
            height: "100%",
            color: "{button_theme.font_theme.color}",
            RouterLink {
                to: cx.props.to,
                container {
                    width: "100%",
                    height: "100%",
                    padding: "15",
                    label {
                        align: "center",
                        height: "100%",
                        width: "100%",
                        content
                    }
                }
            }
        }
    )
}

#[allow(non_snake_case)]
fn NodeInspectorBar(cx: Scope) -> Element {
    render!(
        TabsBar {
            TabButton {
                to: "/elements/style",
                label: "Style"
            }
            TabButton {
                to: "/elements/listeners",
                label: "Listeners"
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn NodeInspectorStyle<'a>(cx: Scope<'a>, node: &'a TreeNode) -> Element<'a> {
    render!(
        container {
            width: "100%",
            height: "50%",
            NodeInspectorBar { }
            ScrollView {
                show_scrollbar: true,
                height: "calc(100% - 35)",
                width: "100%",
                node.state.iter().enumerate().map(|(i, (name, attr))| {
                    match attr {
                        AttributeType::Measure(measure) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: measure.to_string()
                                }
                            }
                        }
                        AttributeType::Measures((a, b, c, d)) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: format!("({a}, {b}, {c}, {d})")
                                }
                            }
                        }
                        AttributeType::Size(size) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: size.to_string()
                                }
                            }
                        }
                        AttributeType::Color(color) => {
                            rsx!{
                                ColorfulProperty {
                                    key: "{i}",
                                    name: "{name}",
                                    color: color
                                }
                            }
                        }
                        AttributeType::Text(text) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: text.to_string()
                                }
                            }
                        }
                        AttributeType::Direction(direction) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: direction.to_string()
                                }
                            }
                        }
                        AttributeType::Display(display) => {
                            rsx!{
                                Property {
                                    key: "{i}",
                                    name: "{name}",
                                    value: display.to_string()
                                }
                            }
                        }
                        AttributeType::Shadow(shadow_settings) => {
                            rsx!{
                                ShadowProperty {
                                    key: "{i}",
                                    name: "{name}",
                                    shadow_settings: shadow_settings
                                }
                            }
                        }
                    }
                })
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn NodeInspectorListeners<'a>(
    cx: Scope<'a>,
    node: &'a TreeNode,
    event_emitter: UnboundedSender<DomEvent>,
) -> Element<'a> {
    render!(
        container {
            width: "100%",
            height: "50%",
            NodeInspectorBar { }
            node.listeners.as_ref().and_then(move|listeners| {
                Some(rsx!{
                    listeners.iter()
                    .map(|listener| {
                        rsx!(
                            EventListener {
                                listener: listener,
                                node: node,
                                event_emitter: event_emitter
                            }
                        )
                    })
                })
            })
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn ClickEventListenerExtra<'a>(
    cx: Scope<'a>,
    listener: &'a str,
    node: &'a TreeNode,
    event_emitter: &'a UnboundedSender<DomEvent>,
) -> Element<'a> {
    let positions = use_state(cx, || (0.0, 0.0));

    let event_handler = move |_: MouseEvent| {
        let event = DomEvent {
            name: listener.to_string(),
            element_id: node.element_id,
            data: DomEventData::Mouse(MouseData::new(
                Point2D::from_lengths(Length::new(0.0), Length::new(0.0)),
                Point2D::from_lengths(Length::new(positions.0), Length::new(positions.1)),
                Some(MouseButton::Left),
            )),
        };
        event_emitter.send(event).ok();
    };

    render!(
        Input {
            onchange: |v: String| {
                positions.set((v.parse::<f64>().unwrap_or(positions.0), positions.1));
            },
            value: "{positions.0}"
        }
        Input {
            onchange: |v: String| {
                positions.set((positions.0, v.parse::<f64>().unwrap_or(positions.1)));
            },
            value: "{positions.1}"
        }
        Button {
            onclick: event_handler,
            label { "Trigger" }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn EventListener<'a>(
    cx: Scope<'a>,
    listener: &'a str,
    node: &'a TreeNode,
    event_emitter: &'a UnboundedSender<DomEvent>,
) -> Element<'a> {
    let (color, extra) = match *listener {
        "mousedown" | "click" => (
            "white",
            Some(rsx! {
                ClickEventListenerExtra {
                    listener: listener,
                    node: node,
                    event_emitter: event_emitter
                }
            }),
        ),
        _ => ("rgb(200, 200, 200)", None),
    };

    render!(
        rect {
            width: "100%",
            height: "50",
            padding: "15",
            direction: "horizontal",
            rect {
                width: "calc(100% - 270)",
                height: "100%",
                display: "center",
                direction: "vertical",
                label {
                    color: "{color}",
                    "{listener}"
                }
            }
            extra
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn Property<'a>(cx: Scope<'a>, name: &'a str, value: String) -> Element<'a> {
    render!(
        container {
            height: "30",
            width: "100%",
            direction: "horizontal",
            padding: "20",
            paragraph {
                width: "100%",
                text {
                    font_size: "15",
                    color: "rgb(71, 180, 240)",
                    "{name}"
                }
                text {
                    font_size: "15",
                    color: "rgb(215, 215, 215)",
                    ": "
                }
                text {
                    font_size: "15",
                    color: "rgb(252,181,172)",
                    "{value}"
                }
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn ColorfulProperty<'a>(cx: Scope<'a>, name: &'a str, color: &'a Color) -> Element<'a> {
    let color = color.to_rgb();
    render!(
        container {
            height: "30",
            width: "100%",
            direction: "horizontal",
            padding: "20",
            label {
                font_size: "15",
                color: "rgb(71, 180, 240)",
                "{name}"
            }
            label {
                font_size: "15",
                color: "rgb(215, 215, 215)",
                ": "
            }
            rect {
                width: "5"
            }
            rect {
                width: "17",
                height: "17",
                radius: "5",
                background: "white",
                padding: "5",
                rect {
                    radius: "3",
                    width: "100%",
                    height: "100%",
                    background: "rgb({color.r}, {color.g}, {color.b})",
                }
            }
            rect {
                width: "5"
            }
            label {
                font_size: "15",
                color: "rgb(252,181,172)",
                "rgb({color.r}, {color.g}, {color.b})"
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn ShadowProperty<'a>(
    cx: Scope<'a>,
    name: &'a str,
    shadow_settings: &'a ShadowSettings,
) -> Element<'a> {
    let color = shadow_settings.color.to_rgb();
    render!(
        container {
            height: "30",
            width: "100%",
            direction: "horizontal",
            padding: "20",
            paragraph {
                text {
                    font_size: "15",
                    color: "rgb(71, 180, 240)",
                    "{name}"
                }
                text {
                    font_size: "15",
                    color: "rgb(215, 215, 215)",
                    ": "
                }
                text {
                    font_size: "15",
                    color: "rgb(252,181,172)",
                    "{shadow_settings.x} {shadow_settings.y} {shadow_settings.intensity} {shadow_settings.size}"
                }
            }
            rect {
                width: "5"
            }
            rect {
                width: "17",
                height: "17",
                radius: "5",
                background: "white",
                padding: "5",
                rect {
                    radius: "3",
                    width: "100%",
                    height: "100%",
                    background: "rgb({color.r}, {color.g}, {color.b})",
                }
            }
            rect {
                width: "5"
            }
            label {
                font_size: "15",
                color: "rgb(252,181,172)",
                "rgb({color.r}, {color.g}, {color.b})"
            }
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn NodeElement<'a>(
    cx: Scope<'a>,
    node: &'a TreeNode,
    is_selected: bool,
    onselected: EventHandler<'a, &'a TreeNode>,
) -> Element<'a> {
    let text_color = use_state(cx, || "white");

    let mut color = *text_color.get();
    let margin_left = (node.height * 10) as f32 + 16.5;
    let mut background = "transparent";
    let listeners = node
        .listeners
        .as_ref()
        .map(|listeners| format!("({})", listeners.len()))
        .unwrap_or("".to_owned());

    if *is_selected {
        color = "white";
        background = "rgb(100, 100, 100)";
    };

    render!(
        rect {
            radius: "7",
            padding: "10",
            background: background,
            width: "100%",
            height: "27",
            scroll_x: "{margin_left}",
            onmousedown: |_| onselected.call(node),
            onmouseover: move |_| {
                text_color.set("rgb(150, 150, 150)");
            },
            onmouseleave: move |_| {
                text_color.set("white");
            },
            label {
                font_size: "14",
                color: "{color}",
                "{node.tag} #{node.id.0} {listeners}"
            }
        }
    )
}
