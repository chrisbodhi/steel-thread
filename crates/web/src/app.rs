use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::plate_form::{PlateForm, SubmitPlate};
use crate::components::response_panel::ResponsePanel;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Brighton Actuation Systems"/>
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
            <h1 class="text-red-500">Actuator Plate Configurator</h1>
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
