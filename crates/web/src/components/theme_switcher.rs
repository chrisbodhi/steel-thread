use leptos::prelude::*;

use crate::theme::Theme;

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    let theme = expect_context::<ReadSignal<Theme>>();
    let set_theme = expect_context::<WriteSignal<Theme>>();

    view! {
        <div class="theme-switcher">
            <label for="theme-select">"Theme:"</label>
            <select
                id="theme-select"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    let new_theme = match value.as_str() {
                        "brighton" => Theme::Brighton,
                        "industrial" => Theme::Industrial,
                        "classic" => Theme::Classic,
                        _ => Theme::Brighton,
                    };
                    set_theme.set(new_theme);
                }
            >
                {Theme::all().into_iter().map(|t| {
                    view! {
                        <option
                            value=t.as_str()
                            selected=move || theme.get() == t
                        >
                            {t.display_name()}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
