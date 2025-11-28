use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::plate_form::PlateForm;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/web.css"/>
        <Title text="Actuator Plate Configurator???"/>
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
    view! {
        <div class="container">
            <h1 class="text-red-500">Actuator Plate Configurator!</h1>
                <div class="container columns-2xs">
                    <img src="https://i.pinimg.com/originals/35/c0/2b/35c02b534cdbacbea92ae64ee3fe0a1d.png" alt="Cat CAD" />
                    <PlateForm/>
                </div>
        </div>
    }
}
