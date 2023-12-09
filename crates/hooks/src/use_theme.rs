#[doc(hidden)]
pub use ::core::default::Default;
#[doc(hidden)]
pub use ::paste::paste;
#[doc(hidden)]
pub use ::std::borrow::Cow;
use dioxus_core::ScopeState;
use dioxus_hooks::{use_shared_state, use_shared_state_provider, UseSharedState};

/// Provide a custom [`Theme`].
pub fn use_init_theme(cx: &ScopeState, theme: Theme) {
    use_shared_state_provider(cx, || theme);
}

/// Provide the default [`Theme`].
pub fn use_init_default_theme(cx: &ScopeState) {
    use_shared_state_provider(cx, Theme::default);
}

/// Subscribe to [`Theme`] changes.
pub fn use_theme(cx: &ScopeState) -> &UseSharedState<Theme> {
    use_shared_state::<Theme>(cx).unwrap()
}

/// Subscribe to [`Theme`] changes, default theme will be used if there is no provided [`Theme`].
///
/// Primarily used by built-in components that have no control of whether they will inherit a [`Theme`] or not.
pub fn use_get_theme(cx: &ScopeState) -> Theme {
    use_shared_state::<Theme>(cx)
        .map(|v| v.read().clone())
        .unwrap_or_default()
}

/// Utility macro.
#[macro_export]
#[doc(hidden)]
macro_rules! empty {
    () => {};
}

/// Example usage:
///
/// ```rust,ignore
/// define_theme! {
///     %[component]
///     pub Test<'a> {
///         %[cows]
///         cow_string: str,
///         %[borrowed]
///         borrowed_data: &'a Foo,
///         %[owned]
///         owned_data: Bar,
///         %[subthemes],
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_theme {
    (
        $(#[$attrs:meta])*
        $(%[component$($component_attr_control:tt)?])?
        $vis:vis $name:ident $(<$lifetime:lifetime>)? {
            $(
                %[cows]
                $(
                    $(#[$cow_field_attrs:meta])*
                    $cow_field_name:ident: $cow_field_ty:ty,
                )*
            )?
            $(
                %[borrowed]
                $(
                    $(#[$borrowed_field_attrs:meta])*
                    $borrowed_field_name:ident: $borrowed_field_ty:ty,
                )*
            )?
            $(
                %[owned]
                $(
                    $(#[$owned_field_attrs:meta])*
                    $owned_field_name:ident: $owned_field_ty:ty,
                )*
            )?
            $(
                %[subthemes]
                $(
                    $(#[$subtheme_field_attrs:meta])*
                    $subtheme_field_name:ident: $subtheme_field_ty_name:ident $(<$subtheme_field_ty_lifetime:lifetime>)?,
                )*
            )?
    }) => {
        $crate::empty!($($($component_attr_control)?)?);
        $crate::paste! {
            #[derive(Default, Clone, Debug, PartialEq, Eq)]
            #[doc = "You can use this to change a theme for only one component, with the `theme` property."]
            $(#[$attrs])*
            $vis struct [<$name ThemeWith>] $(<$lifetime>)? {
                $($(
                    $(#[$borrowed_field_attrs])*
                    pub $borrowed_field_name: Option<$borrowed_field_ty>,
                )*)?
                $($(
                    $(#[$owned_field_attrs])*
                    pub $owned_field_name: Option<$owned_field_ty>,
                )*)?
                $($(
                    $(#[$subtheme_field_attrs])*
                    pub $subtheme_field_name: Option< [<$subtheme_field_ty_name With>] $(<$subtheme_field_ty_lifetime>)? >,
                )*)?
                $($(
                    $(#[$cow_field_attrs])*
                    pub $cow_field_name: Option<$crate::Cow<'static, $cow_field_ty>>,
                )*)?
            }

            #[derive(Clone, Debug, PartialEq, Eq)]
            $(#[doc = "Theming properties for the `" $name "` component."] $($component_attr_control)?)?
            $(#[$attrs])*
            $vis struct [<$name Theme>] $(<$lifetime>)? {
                $($(
                    $(#[$borrowed_field_attrs])*
                    pub $borrowed_field_name: $borrowed_field_ty,
                )*)?
                $($(
                    $(#[$owned_field_attrs])*
                    pub $owned_field_name: $owned_field_ty,
                )*)?
                $($(
                    $(#[$subtheme_field_attrs])*
                    pub $subtheme_field_name: $subtheme_field_ty_name $(<$subtheme_field_ty_lifetime>)?,
                )*)?
                $($(
                    $(#[$cow_field_attrs])*
                    pub $cow_field_name: $crate::Cow<'static, $cow_field_ty>,
                )*)?
            }

            impl $(<$lifetime>)? [<$name Theme>] $(<$lifetime>)? {
                #[doc = "Checks each field in `optional` and if it's `Some`, it overwrites the corresponding `self` field."]
                pub fn apply_optional(&mut self, optional: & $($lifetime)? [<$name ThemeWith>]) {
                    $($(
                        if let Some($borrowed_field_name) = optional.$borrowed_field_name {
                            self.$borrowed_field_name = $borrowed_field_name;
                        }
                    )*)?

                    $($(
                        if let Some($owned_field_name) = &optional.$owned_field_name {
                            self.$owned_field_name = $owned_field_name.clone();
                        }
                    )*)?

                    $($(
                        if let Some($subtheme_field_name) = &optional.$subtheme_field_name {
                            self.$subtheme_field_name.apply_optional($subtheme_field_name);
                        }
                    )*)?

                    $($(
                        if let Some($cow_field_name) = &optional.$cow_field_name {
                            self.$cow_field_name = $cow_field_name.clone();
                        }
                    )*)?
                }
            }
        }
    };
}

/// Create `FooThemeWith` structs without having to deal with the verbose syntax.
///
/// # Examples
///
/// Without the macro:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use freya::prelude::*;
/// # fn theme_with_example_no_macro(cx: Scope) -> Element {
/// render! {
///     Button {
///         theme: ButtonThemeWith {
///             background: "blue".into(),
///             font_theme: FontThemeWith {
///                 color: "white".into(),
///                 ..Default::default()
///             }.into(),
///             ..Default::default()
///         }
///     }
/// }
/// # }
/// ```
///
/// With the macro:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use freya::prelude::*;
/// # fn theme_with_example_no_macro(cx: Scope) -> Element {
/// render! {
///     Button {
///         theme: theme_with!(ButtonTheme {
///             background: "blue",
///             font_theme: theme_with!(FontTheme {
///                 color: "white",
///             }),
///         })
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! theme_with {
    ($theme_name:ident {
        $(
            $theme_field_name:ident: $theme_field_val:expr,
        )*
    }) => {
        $crate::paste! {
            [<$theme_name With>] {
                $($theme_field_name: Some($theme_field_val),)*
                ..$crate::Default::default()
            }
        }
    };
}

define_theme! {
    %[component]
    pub Dropdown {
        %[cows]
        desplegable_background: str,
        background_button: str,
        hover_background: str,
        border_fill: str,
        arrow_fill: str,
        %[subthemes]
        font_theme: FontTheme,
    }
}

define_theme! {
    %[component]
    pub DropdownItem {
        %[cows]
        background: str,
        select_background: str,
        hover_background: str,
        %[subthemes]
        font_theme: FontTheme,
    }
}

define_theme! {
    %[component]
    pub Button {
        %[cows]
        background: str,
        hover_background: str,
        border_fill: str,
        margin: str,
        corner_radius: str,
        width: str,
        height: str,
        padding: str,
        %[subthemes]
        font_theme: FontTheme,
    }
}

define_theme! {
    %[component]
    pub Input {
        %[cows]
        background: str,
        hover_background: str,
        border_fill: str,
        %[subthemes]
        font_theme: FontTheme,
    }
}

define_theme! {
    /// Theming properties for Fonts.
    pub Font {
        %[cows]
        color: str,
    }
}

define_theme! {
    %[component]
    pub Switch {
        %[cows]
        background: str,
        thumb_background: str,
        enabled_background: str,
        enabled_thumb_background: str,
    }
}

define_theme! {
    %[component]
    pub Scrollbar {
        %[cows]
        background: str,
        thumb_background: str,
        hover_thumb_background: str,
        active_thumb_background: str,
    }
}

define_theme! {
    %[component]
    pub Body {
        %[cows]
        background: str,
        color: str,
    }
}

define_theme! {
    %[component]
    pub Slider {
        %[cows]
        background: str,
        thumb_background: str,
        thumb_inner_background: str,
    }
}

define_theme! {
    %[component]
    pub Tooltip {
        %[cows]
        background: str,
        color: str,
        border_fill: str,
    }
}

define_theme! {
    %[component]
    pub ExternalLink {
        %[cows]
        highlight_color: str,
    }
}

define_theme! {
    %[component]
    pub Accordion {
        %[cows]
        color: str,
        background: str,
        border_fill: str,
    }
}

define_theme! {
    %[component]
    pub Loader {
        %[cows]
        primary_color: str,
        secondary_color: str,
    }
}

define_theme! {
    %[component]
    pub ProgressBar {
        %[cows]
        color: str,
        background: str,
        progress_background: str,
    }
}

define_theme! {
    %[component]
    pub Table {
        %[cows]
        background: str,
        arrow_fill: str,
        alternate_row_background: str,
        row_background: str,
        divider_fill: str,
        %[subthemes]
        font_theme: FontTheme,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub body: BodyTheme,
    pub button: ButtonTheme,
    pub switch: SwitchTheme,
    pub scrollbar: ScrollbarTheme,
    pub slider: SliderTheme,
    pub tooltip: TooltipTheme,
    pub external_link: ExternalLinkTheme,
    pub dropdown: DropdownTheme,
    pub dropdown_item: DropdownItemTheme,
    pub accordion: AccordionTheme,
    pub loader: LoaderTheme,
    pub progress_bar: ProgressBarTheme,
    pub table: TableTheme,
    pub input: InputTheme,
}

impl Default for Theme {
    fn default() -> Self {
        LIGHT_THEME
    }
}

pub const LIGHT_THEME: Theme = Theme {
    name: "light",
    body: BodyTheme {
        background: Cow::Borrowed("white"),
        color: Cow::Borrowed("black"),
    },
    slider: SliderTheme {
        background: Cow::Borrowed("rgb(210, 210, 210)"),
        thumb_background: Cow::Borrowed("rgb(210, 210, 210)"),
        thumb_inner_background: Cow::Borrowed("rgb(103, 80, 164)"),
    },
    button: ButtonTheme {
        background: Cow::Borrowed("rgb(245, 245, 245)"),
        hover_background: Cow::Borrowed("rgb(235, 235, 235)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("rgb(10, 10, 10)"),
        },
        border_fill: Cow::Borrowed("rgb(210, 210, 210)"),
        padding: Cow::Borrowed("8 16"),
        margin: Cow::Borrowed("4"),
        corner_radius: Cow::Borrowed("8"),
        width: Cow::Borrowed("auto"),
        height: Cow::Borrowed("auto"),
    },
    input: InputTheme {
        background: Cow::Borrowed("rgb(245, 245, 245)"),
        hover_background: Cow::Borrowed("rgb(235, 235, 235)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("rgb(10, 10, 10)"),
        },
        border_fill: Cow::Borrowed("rgb(210, 210, 210)"),
    },
    switch: SwitchTheme {
        background: Cow::Borrowed("rgb(121, 116, 126)"),
        thumb_background: Cow::Borrowed("rgb(231, 224, 236)"),
        enabled_background: Cow::Borrowed("rgb(103, 80, 164)"),
        enabled_thumb_background: Cow::Borrowed("rgb(234, 221, 255)"),
    },
    scrollbar: ScrollbarTheme {
        background: Cow::Borrowed("rgb(225, 225, 225)"),
        thumb_background: Cow::Borrowed("rgb(135, 135, 135)"),
        hover_thumb_background: Cow::Borrowed("rgb(115, 115, 115)"),
        active_thumb_background: Cow::Borrowed("rgb(95, 95, 95)"),
    },
    tooltip: TooltipTheme {
        background: Cow::Borrowed("rgb(245, 245, 245)"),
        color: Cow::Borrowed("rgb(25,25,25)"),
        border_fill: Cow::Borrowed("rgb(210, 210, 210)"),
    },
    external_link: ExternalLinkTheme {
        highlight_color: Cow::Borrowed("rgb(43,106,208)"),
    },
    dropdown: DropdownTheme {
        desplegable_background: Cow::Borrowed("white"),
        background_button: Cow::Borrowed("rgb(245, 245, 245)"),
        hover_background: Cow::Borrowed("rgb(235, 235, 235)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("rgb(10, 10, 10)"),
        },
        border_fill: Cow::Borrowed("rgb(210, 210, 210)"),
        arrow_fill: Cow::Borrowed("rgb(40, 40, 40)"),
    },
    dropdown_item: DropdownItemTheme {
        background: Cow::Borrowed("white"),
        select_background: Cow::Borrowed("rgb(240, 240, 240)"),
        hover_background: Cow::Borrowed("rgb(220, 220, 220)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("rgb(10, 10, 10)"),
        },
    },
    accordion: AccordionTheme {
        color: Cow::Borrowed("black"),
        background: Cow::Borrowed("rgb(245, 245, 245)"),
        border_fill: Cow::Borrowed("rgb(210, 210, 210)"),
    },
    loader: LoaderTheme {
        primary_color: Cow::Borrowed("rgb(50, 50, 50)"),
        secondary_color: Cow::Borrowed("rgb(150, 150, 150)"),
    },
    progress_bar: ProgressBarTheme {
        color: Cow::Borrowed("white"),
        background: Cow::Borrowed("rgb(210, 210, 210)"),
        progress_background: Cow::Borrowed("rgb(103, 80, 164)"),
    },
    table: TableTheme {
        font_theme: FontTheme {
            color: Cow::Borrowed("black"),
        },
        background: Cow::Borrowed("white"),
        arrow_fill: Cow::Borrowed("rgb(40, 40, 40)"),
        row_background: Cow::Borrowed("transparent"),
        alternate_row_background: Cow::Borrowed("rgb(240, 240, 240)"),
        divider_fill: Cow::Borrowed("rgb(200, 200, 200)"),
    },
};

pub const DARK_THEME: Theme = Theme {
    name: "dark",
    body: BodyTheme {
        background: Cow::Borrowed("rgb(25, 25, 25)"),
        color: Cow::Borrowed("white"),
    },
    slider: SliderTheme {
        background: Cow::Borrowed("rgb(60, 60, 60)"),
        thumb_background: Cow::Borrowed("rgb(60, 60, 60)"),
        thumb_inner_background: Cow::Borrowed("rgb(255, 95, 0)"),
    },
    button: ButtonTheme {
        background: Cow::Borrowed("rgb(35, 35, 35)"),
        hover_background: Cow::Borrowed("rgb(45, 45, 45)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
        border_fill: Cow::Borrowed("rgb(80, 80, 80)"),
        padding: Cow::Borrowed("8 16"),
        margin: Cow::Borrowed("4"),
        corner_radius: Cow::Borrowed("8"),
        width: Cow::Borrowed("auto"),
        height: Cow::Borrowed("auto"),
    },
    input: InputTheme {
        background: Cow::Borrowed("rgb(35, 35, 35)"),
        hover_background: Cow::Borrowed("rgb(45, 45, 45)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
        border_fill: Cow::Borrowed("rgb(80, 80, 80)"),
    },
    switch: SwitchTheme {
        background: Cow::Borrowed("rgb(60, 60, 60)"),
        thumb_background: Cow::Borrowed("rgb(200, 200, 200)"),
        enabled_background: Cow::Borrowed("rgb(255, 95, 0)"),
        enabled_thumb_background: Cow::Borrowed("rgb(234, 221, 255)"),
    },
    scrollbar: ScrollbarTheme {
        background: Cow::Borrowed("rgb(35, 35, 35)"),
        thumb_background: Cow::Borrowed("rgb(100, 100, 100)"),
        hover_thumb_background: Cow::Borrowed("rgb(120, 120, 120)"),
        active_thumb_background: Cow::Borrowed("rgb(140, 140, 140)"),
    },
    tooltip: TooltipTheme {
        background: Cow::Borrowed("rgb(35,35,35)"),
        color: Cow::Borrowed("rgb(240,240,240)"),
        border_fill: Cow::Borrowed("rgb(80, 80, 80)"),
    },
    external_link: ExternalLinkTheme {
        highlight_color: Cow::Borrowed("rgb(43,106,208)"),
    },
    dropdown: DropdownTheme {
        desplegable_background: Cow::Borrowed("rgb(25, 25, 25)"),
        background_button: Cow::Borrowed("rgb(35, 35, 35)"),
        hover_background: Cow::Borrowed("rgb(45, 45, 45)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
        border_fill: Cow::Borrowed("rgb(80, 80, 80)"),
        arrow_fill: Cow::Borrowed("rgb(40, 40, 40)"),
    },
    dropdown_item: DropdownItemTheme {
        background: Cow::Borrowed("rgb(35, 35, 35)"),
        select_background: Cow::Borrowed("rgb(80, 80, 80)"),
        hover_background: Cow::Borrowed("rgb(55, 55, 55)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
    },
    accordion: AccordionTheme {
        color: Cow::Borrowed("white"),
        background: Cow::Borrowed("rgb(60, 60, 60)"),
        border_fill: Cow::Borrowed("rgb(80, 80, 80)"),
    },
    loader: LoaderTheme {
        primary_color: Cow::Borrowed("rgb(150, 150, 150)"),
        secondary_color: Cow::Borrowed("rgb(255, 255, 255)"),
    },
    progress_bar: ProgressBarTheme {
        color: Cow::Borrowed("white"),
        background: Cow::Borrowed("rgb(60, 60, 60)"),
        progress_background: Cow::Borrowed("rgb(255, 95, 0)"),
    },
    table: TableTheme {
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
        background: Cow::Borrowed("rgb(25, 25, 25)"),
        arrow_fill: Cow::Borrowed("rgb(150, 150, 150)"),
        row_background: Cow::Borrowed("transparent"),
        alternate_row_background: Cow::Borrowed("rgb(50, 50, 50)"),
        divider_fill: Cow::Borrowed("rgb(100, 100, 100)"),
    },
};
