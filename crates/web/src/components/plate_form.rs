use domain::{ActuatorPlate, Millimeters};
use leptos::prelude::*;

#[server]
pub async fn submit_plate(
    bolt_spacing: u16,
    bolt_diameter: u16,
    bracket_height: u16,
    pin_diameter: u16,
    plate_thickness: u16,
) -> Result<String, ServerFnError> {
    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(bolt_spacing),
        bolt_diameter: Millimeters(bolt_diameter),
        bracket_height: Millimeters(bracket_height),
        pin_diameter: Millimeters(pin_diameter),
        plate_thickness: Millimeters(plate_thickness),
    };

    if let Err(e) = validation::validate(&plate) {
        return Err(ServerFnError::new(e.to_string()));
    }

    Ok("Plate submitted successfully!".to_string())
}

#[component]
pub fn PlateForm() -> impl IntoView {
    let submit_action = ServerAction::<SubmitPlate>::new();

    // Field error states
    let (bolt_spacing_error, set_bolt_spacing_error) = signal(None::<String>);
    let (bolt_diameter_error, set_bolt_diameter_error) = signal(None::<String>);
    let (bracket_height_error, set_bracket_height_error) = signal(None::<String>);
    let (pin_diameter_error, set_pin_diameter_error) = signal(None::<String>);
    let (plate_thickness_error, set_plate_thickness_error) = signal(None::<String>);

    let validate_field = move |value: &str, validator: fn(u16) -> Result<(), validation::PlateValidationError>| {
        match value.parse::<u16>() {
            Ok(val) => match validator(val) {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            },
            Err(_) if value.is_empty() => Some("This field is required".to_string()),
            Err(_) => Some("Invalid number".to_string()),
        }
    };

    view! {
        <ActionForm action=submit_action>
            <div class="form-group">
                <label for="bolt_spacing">"Bolt Spacing (mm):"</label>
                <input
                    type="number"
                    id="bolt_spacing"
                    name="bolt_spacing"
                    value="60"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_bolt_spacing_error.set(validate_field(&value, validation::validate_bolt_spacing));
                    }
                    required
                    class:error=move || bolt_spacing_error.get().is_some()
                />
                {move || bolt_spacing_error.get().map(|err| view! {
                    <span class="error-message">{err}</span>
                })}
            </div>

            <div class="form-group">
                <label for="bolt_diameter">"Bolt Diameter (mm):"</label>
                <input
                    type="number"
                    id="bolt_diameter"
                    name="bolt_diameter"
                    value="10"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_bolt_diameter_error.set(validate_field(&value, validation::validate_bolt_diameter));
                    }
                    required
                    class:error=move || bolt_diameter_error.get().is_some()
                />
                {move || bolt_diameter_error.get().map(|err| view! {
                    <span class="error-message">{err}</span>
                })}
            </div>

            <div class="form-group">
                <label for="bracket_height">"Bracket Height (mm):"</label>
                <input
                    type="number"
                    id="bracket_height"
                    name="bracket_height"
                    value="40"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_bracket_height_error.set(validate_field(&value, validation::validate_bracket_height));
                    }
                    required
                    class:error=move || bracket_height_error.get().is_some()
                />
                {move || bracket_height_error.get().map(|err| view! {
                    <span class="error-message">{err}</span>
                })}
            </div>

            <div class="form-group">
                <label for="pin_diameter">"Pin Diameter (mm):"</label>
                <input
                    type="number"
                    id="pin_diameter"
                    name="pin_diameter"
                    value="10"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_pin_diameter_error.set(validate_field(&value, validation::validate_pin_diameter));
                    }
                    required
                    class:error=move || pin_diameter_error.get().is_some()
                />
                {move || pin_diameter_error.get().map(|err| view! {
                    <span class="error-message">{err}</span>
                })}
            </div>

            <div class="form-group">
                <label for="plate_thickness">"Plate Thickness (mm):"</label>
                <input
                    type="number"
                    id="plate_thickness"
                    name="plate_thickness"
                    value="8"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_plate_thickness_error.set(validate_field(&value, validation::validate_plate_thickness));
                    }
                    required
                    class:error=move || plate_thickness_error.get().is_some()
                />
                {move || plate_thickness_error.get().map(|err| view! {
                    <span class="error-message">{err}</span>
                })}
            </div>

            <button
                type="submit"
                disabled=move || {
                    submit_action.pending().get()
                    || bolt_spacing_error.get().is_some()
                    || bolt_diameter_error.get().is_some()
                    || bracket_height_error.get().is_some()
                    || pin_diameter_error.get().is_some()
                    || plate_thickness_error.get().is_some()
                }
            >
                {move || if submit_action.pending().get() { "Submitting..." } else { "Submit Plate" }}
            </button>

            {move || {
                submit_action.value().get().map(|result| {
                    match result {
                        Ok(msg) => view! { <div class="response-message success">{msg}</div> }.into_any(),
                        Err(e) => view! { <div class="response-message error">{e.to_string()}</div> }.into_any(),
                    }
                })
            }}
        </ActionForm>
    }
}
