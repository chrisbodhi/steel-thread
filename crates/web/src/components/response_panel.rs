use leptos::prelude::*;

use crate::components::plate_form::SubmitPlate;

#[component]
pub fn ResponsePanel(
    #[prop(into)] action: ServerAction<SubmitPlate>,
) -> impl IntoView {
    let (order_count, set_order_count) = signal(1u32);

    let increment = move |_| set_order_count.update(|count| *count += 1);
    let decrement = move |_| set_order_count.update(|count| {
        if *count > 1 {
            *count -= 1;
        }
    });

    view! {
        <div class="response-panel">
            {move || {
                action.value().get().and_then(|result| {
                    match result {
                        Ok(msg) => Some(view! {
                            <div class="response-content">
                                <div class="success-message">{msg}</div>

                                <div class="order-section">
                                    <h3>"Order Your Plate"</h3>

                                    <div class="quantity-control">
                                        <button
                                            type="button"
                                            class="quantity-btn"
                                            on:click=decrement
                                            disabled=move || order_count.get() <= 1
                                        >
                                            "-"
                                        </button>
                                        <span class="quantity-display">{move || order_count.get()}</span>
                                        <button
                                            type="button"
                                            class="quantity-btn"
                                            on:click=increment
                                        >
                                            "+"
                                        </button>
                                    </div>

                                    <button type="button" class="order-btn">
                                        {move || format!("Order {} Plate{}", order_count.get(), if order_count.get() == 1 { "" } else { "s" })}
                                    </button>
                                </div>

                                <div class="download-section">
                                    <button type="button" class="download-btn">
                                        "📥 Download STEP File"
                                    </button>
                                </div>
                            </div>
                        }.into_any()),
                        Err(e) => Some(view! {
                            <div class="error-message">
                                <strong>"Error: "</strong>
                                {e.to_string()}
                            </div>
                        }.into_any()),
                    }
                })
            }}
        </div>
    }
}
