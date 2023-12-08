use dioxus::prelude::*;
use freya_hooks::{use_init_theme, Theme};

/// [`ThemeProvider`] component properties.
#[derive(Props)]
pub struct ThemeProviderProps<'a> {
    /// Theme to provide.
    #[props(optional)]
    pub theme: Option<Theme>,
    /// Inner children to provide a Theme to.
    pub children: Element<'a>,
}

/// Provides a `Theme` for all its children.
///
/// # Props
/// See [`ThemeProviderProps`]
///
#[allow(non_snake_case)]
pub fn ThemeProvider<'a>(cx: Scope<'a, ThemeProviderProps<'a>>) -> Element<'a> {
    use_init_theme(cx, cx.props.theme.clone().unwrap_or_default());

    render!(&cx.props.children)
}

macro_rules! get_theme {
    ($cx:expr, $theme_prop:expr, $theme_name:ident) => {{
        let mut theme = ::freya_hooks::use_get_theme($cx).$theme_name;

        if let Some(theme_override) = $theme_prop {
            theme.apply_optional(theme_override);
        }

        theme
    }};
}

pub(crate) use get_theme;
