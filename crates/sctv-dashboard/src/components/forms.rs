//! Form components with modern, bold styling.
//!
//! Features clean input designs with geometric accents
//! and clear visual feedback states.

// Workaround for Leptos 0.7 not forwarding lint attributes to the __Component inner
// function. See: https://github.com/leptos-rs/leptos/issues/3771
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

use super::icons::{ChevronDownIcon, SearchIcon};

/// Text input with label and optional error state.
#[component]
pub fn TextInput(
    #[prop(into)] name: String,
    #[prop(into)] label: String,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] value: Option<String>,
    #[prop(optional)] error: Option<String>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let input_class = format!(
        "text-input__field {}",
        if error.is_some() {
            "text-input__field--error"
        } else {
            ""
        }
    );

    view! {
        <div class="text-input">
            <label class="text-input__label" for=name.clone()>
                {label}
            </label>
            <input
                type="text"
                id=name.clone()
                name=name
                class=input_class
                placeholder=placeholder.unwrap_or_default()
                value=value.unwrap_or_default()
                disabled=disabled
            />
            {error.map(|e| view! { <span class="text-input__error">{e}</span> })}
        </div>
    }
}

/// Search input with icon.
#[component]
pub fn SearchInput(
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] value: Option<String>,
) -> impl IntoView {
    view! {
        <div class="search-input">
            <SearchIcon/>
            <input
                type="search"
                class="search-input__field"
                placeholder=placeholder.unwrap_or_else(|| "Search...".to_string())
                value=value.unwrap_or_default()
            />
        </div>
    }
}

/// Select dropdown with custom styling.
#[component]
#[allow(clippy::needless_pass_by_value)]
pub fn Select(
    #[prop(into)] name: String,
    #[prop(into)] label: String,
    #[prop(into)] options: Vec<(String, String)>,
    #[prop(optional)] value: Option<String>,
) -> impl IntoView {
    // Prop type Option<String> is part of the Leptos component API surface and cannot
    // change to Option<&str> without breaking callers. The value is consumed only as a
    // borrow in the .as_deref() comparison when computing options_with_selected.
    // Pre-mark each option as selected; `value` is compared by reference.
    let options_with_selected: Vec<(String, String, bool)> = options
        .into_iter()
        .map(|(opt_value, opt_label)| {
            let selected = value.as_deref() == Some(opt_value.as_str());
            (opt_value, opt_label, selected)
        })
        .collect();
    view! {
        <div class="select">
            <label class="select__label" for=name.clone()>
                {label}
            </label>
            <div class="select__wrapper">
                <select
                    id=name.clone()
                    name=name
                    class="select__field"
                >
                    {options_with_selected
                        .into_iter()
                        .map(|(opt_value, opt_label, selected)| {
                            view! {
                                <option value=opt_value selected=selected>
                                    {opt_label}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
                <ChevronDownIcon/>
            </div>
        </div>
    }
}

/// Toggle switch component.
#[component]
pub fn Toggle(
    #[prop(into)] name: String,
    #[prop(into)] label: String,
    #[prop(optional)] checked: bool,
) -> impl IntoView {
    let (is_checked, set_checked) = signal(checked);

    view! {
        <label class="toggle">
            <input
                type="checkbox"
                name=name
                class="toggle__input"
                checked=is_checked
                on:change=move |ev| {
                    let new_value = event_target_checked(&ev);
                    set_checked.set(new_value);
                }
            />
            <span class="toggle__slider"></span>
            <span class="toggle__label">{label}</span>
        </label>
    }
}

/// Checkbox component.
#[component]
pub fn Checkbox(
    #[prop(into)] name: String,
    #[prop(into)] label: String,
    #[prop(optional)] checked: bool,
) -> impl IntoView {
    view! {
        <label class="checkbox">
            <input
                type="checkbox"
                name=name
                class="checkbox__input"
                checked=checked
            />
            <span class="checkbox__mark"></span>
            <span class="checkbox__label">{label}</span>
        </label>
    }
}

/// Textarea component.
#[component]
pub fn Textarea(
    #[prop(into)] name: String,
    #[prop(into)] label: String,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] value: Option<String>,
    #[prop(optional)] rows: Option<u32>,
) -> impl IntoView {
    view! {
        <div class="textarea">
            <label class="textarea__label" for=name.clone()>
                {label}
            </label>
            <textarea
                id=name.clone()
                name=name
                class="textarea__field"
                placeholder=placeholder.unwrap_or_default()
                rows=rows.unwrap_or(4)
            >
                {value.unwrap_or_default()}
            </textarea>
        </div>
    }
}

/// Button variants.
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Primary => "btn btn--primary",
            Self::Secondary => "btn btn--secondary",
            Self::Ghost => "btn btn--ghost",
            Self::Danger => "btn btn--danger",
        }
    }
}

/// Button sizes.
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Small => "btn--sm",
            Self::Medium => "btn--md",
            Self::Large => "btn--lg",
        }
    }
}

/// Button component with variants.
#[component]
pub fn Button(
    children: Children,
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let class_name = format!("{} {}", variant.class(), size.class());

    view! {
        <button
            class=class_name
            disabled=disabled
        >
            {children()}
        </button>
    }
}

/// Icon button for compact actions.
#[component]
pub fn IconButton(
    children: Children,
    #[prop(into)] label: String,
    #[prop(optional)] variant: ButtonVariant,
) -> impl IntoView {
    view! {
        <button
            class=format!("icon-btn {}", variant.class())
            aria-label=label
        >
            {children()}
        </button>
    }
}

/// Form field group for organizing related inputs.
#[component]
pub fn FieldGroup(#[prop(optional)] label: Option<String>, children: Children) -> impl IntoView {
    view! {
        <fieldset class="field-group">
            {label.map(|l| view! { <legend class="field-group__legend">{l}</legend> })}
            <div class="field-group__content">{children()}</div>
        </fieldset>
    }
}

/// Form actions container.
#[component]
pub fn FormActions(children: Children) -> impl IntoView {
    view! {
        <div class="form-actions">{children()}</div>
    }
}
