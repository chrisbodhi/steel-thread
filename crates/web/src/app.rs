use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::plate_form::{PlateForm, SubmitPlate};
use crate::components::response_panel::ResponsePanel;
use crate::components::theme_switcher::ThemeSwitcher;
use crate::theme::Theme;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Theme state - provide context for entire app
    let (theme, set_theme) = signal(Theme::default());
    provide_context(theme);
    provide_context(set_theme);

    view! {
        <Title text="Brighton Actuation Systems"/>
        <Body attr:data-theme=move || theme.get().as_str()/>
        <Router>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let submit_action = ServerAction::<SubmitPlate>::new();

    view! {
        <div class="container">
            <div class="header">
                <h1 class="text-red-500">Actuator Plate Configurator</h1>
                <ThemeSwitcher/>
            </div>
            <div class="panel-grid">
                <div class="panel">
                    <img src="https://i.pinimg.com/originals/35/c0/2b/35c02b534cdbacbea92ae64ee3fe0a1d.png" alt="Cat CAD" />
                </div>
                <div class="panel">
                    <PlateForm action=submit_action/>
                </div>
                <div class="panel">
                    <ResponsePanel action=submit_action/>
                </div>
            </div>
            <footer class="flex justify-between">
                <div>"©" 2025 Brighton Actuation Systems</div>
                <div>Made with "🧑‍🏭" in PGH. AMDG.</div>
            </footer>
        </div>
    }
}
