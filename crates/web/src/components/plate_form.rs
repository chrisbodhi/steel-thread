use domain::{ActuatorPlate, Millimeters};
use leptos::prelude::*;

#[component]
pub fn PlateForm() -> impl IntoView {
    let (bolt_spacing, set_bolt_spacing) = signal(String::from("60"));
    let (bolt_diameter, set_bolt_diameter) = signal(String::from("10"));
    let (bracket_height, set_bracket_height) = signal(String::from("40"));
    let (pin_diameter, set_pin_diameter) = signal(String::from("10"));
    let (plate_thickness, set_plate_thickness) = signal(String::from("8"));
    let (response_message, set_response_message) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(false);

    let submit_plate = Action::new_local(move |_: &()| {
        let bolt_spacing_val = bolt_spacing.get();
        let bolt_diameter_val = bolt_diameter.get();
        let bracket_height_val = bracket_height.get();
        let pin_diameter_val = pin_diameter.get();
        let plate_thickness_val = plate_thickness.get();

        async move {
            set_is_loading.set(true);
            set_response_message.set(None);

            // Parse inputs
            let bs = bolt_spacing_val.parse::<u16>().ok();
            let bd = bolt_diameter_val.parse::<u16>().ok();
            let bh = bracket_height_val.parse::<u16>().ok();
            let pd = pin_diameter_val.parse::<u16>().ok();
            let pt = plate_thickness_val.parse::<u16>().ok();

            if bs.is_none() || bd.is_none() || bh.is_none() || pd.is_none() || pt.is_none() {
                set_is_loading.set(false);
                set_response_message.set(Some("Invalid input values".to_string()));
                return;
            }

            let plate = ActuatorPlate {
                bolt_spacing: Millimeters(bs.unwrap()),
                bolt_diameter: Millimeters(bd.unwrap()),
                bracket_height: Millimeters(bh.unwrap()),
                pin_diameter: Millimeters(pd.unwrap()),
                plate_thickness: Millimeters(pt.unwrap()),
            };

            // Make API call
            let response = reqwest::Client::new()
                .post("/api/plate")
                .json(&plate)
                .send()
                .await;

            set_is_loading.set(false);

            match response {
                Ok(resp) if resp.status().is_success() => {
                    set_response_message.set(Some("Plate submitted successfully!".to_string()));
                }
                Ok(resp) => {
                    let status = resp.status();
                    set_response_message.set(Some(format!("Error: {}", status)));
                }
                Err(e) => {
                    set_response_message.set(Some(format!("Request failed: {}", e)));
                }
            }
        }
    });

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            submit_plate.dispatch(());
        }>
            <div class="form-group">
                <label for="bolt-spacing">Bolt Spacing (mm):</label>
                <input
                    type="number"
                    id="bolt-spacing"
                    name="bolt-spacing"
                    prop:value=bolt_spacing
                    on:input=move |ev| set_bolt_spacing.set(event_target_value(&ev))
                    required
                />
            </div>

            <div class="form-group">
                <label for="bolt-diameter">"Bolt Diameter (mm):"</label>
                <input
                    type="number"
                    id="bolt-diameter"
                    name="bolt-diameter"
                    prop:value=bolt_diameter
                    on:input=move |ev| set_bolt_diameter.set(event_target_value(&ev))
                    required
                />
            </div>

            <div class="form-group">
                <label for="bracket-height">"Bracket Height (mm):"</label>
                <input
                    type="number"
                    id="bracket-height"
                    name="bracket-height"
                    prop:value=bracket_height
                    on:input=move |ev| set_bracket_height.set(event_target_value(&ev))
                    required
                />
            </div>

            <div class="form-group">
                <label for="pin-diameter">"Pin Diameter (mm):"</label>
                <input
                    type="number"
                    id="pin-diameter"
                    name="pin-diameter"
                    prop:value=pin_diameter
                    on:input=move |ev| set_pin_diameter.set(event_target_value(&ev))
                    required
                />
            </div>

            <div class="form-group">
                <label for="plate-thickness">"Plate Thickness (mm):"</label>
                <input
                    type="number"
                    id="plate-thickness"
                    name="plate-thickness"
                    prop:value=plate_thickness
                    on:input=move |ev| set_plate_thickness.set(event_target_value(&ev))
                    required
                />
            </div>

            <button type="submit" disabled=move || is_loading.get()>
                {move || if is_loading.get() { "Submitting..." } else { "Submit Plate" }}
            </button>

            {move || {
                response_message.get().map(|msg| {
                    view! { <div class="response-message">{msg}</div> }
                })
            }}
        </form>
    }
}
