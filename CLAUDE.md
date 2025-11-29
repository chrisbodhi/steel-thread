# Claude Development Guide

This document provides context and guidelines for AI assistants (like Claude) working on this codebase.

## Project Overview

This is a Leptos-based web application for actuator plate configuration with server-side rendering (SSR), client-side hydration, and real-time validation. The project follows an API-first approach with both REST endpoints and Leptos server functions.

## Technology Stack

### Core Framework & Versions
- **Rust**: 1.91.1
- **Leptos**: 0.8 (SSR framework with reactive UI)
- **Axum**: 0.8 (HTTP server)
- **Tokio**: 1.42 (async runtime)
- **Serde**: 1.0 (serialization)
- **TailwindCSS**: 4.1 

### Build Tools
- **cargo-leptos**: Required for building and running the SSR application
- Use `cargo leptos watch` for development with hot reload
- Use `cargo leptos build` for production builds

## Architecture

### Crate Structure

```
crates/
├── domain/       # Core domain types (ActuatorPlate, Millimeters)
├── validation/   # no_std validation logic (WASM-compatible)
└── web/          # Leptos SSR app + Axum API
```

### Key Architectural Principles

1. **API-First Design**: Always maintain REST API endpoints alongside web UI
   - REST API at `/api/plate` for external consumers
   - Leptos server functions for internal web UI communication
   - Both must use the same validation logic

2. **Shared Validation**:
   - Validation crate is `no_std` for WASM compatibility
   - Same validation code runs on both client (WASM) and server
   - Individual field validators for real-time client-side validation
   - Full plate validation for server-side verification

3. **Idiomatic Leptos Patterns**:
   - Use `#[server]` macro for server functions
   - Use `ServerAction` for type-safe client-server communication
   - Use `ActionForm` for forms (handles submission, progressive enhancement, hydration)
   - Never manually prevent form submission - let `ActionForm` handle it
   - Use reactive signals for state management
   - Use `class:error` syntax for conditional CSS classes

## Common Patterns

### Form Handling

```rust
// Define server function
#[server]
pub async fn submit_data(field: u16) -> Result<String, ServerFnError> {
    // Validation and processing
    Ok("Success".to_string())
}

// In component
let action = ServerAction::<SubmitData>::new();

view! {
    <ActionForm action=action>
        <input type="number" name="field" required />
        <button type="submit">Submit</button>
    </ActionForm>
}
```

### Real-Time Validation

```rust
let (field_error, set_field_error) = signal(None::<String>);

let validate_field = move |value: &str, validator: fn(u16) -> Result<(), Error>| {
    match value.parse::<u16>() {
        Ok(val) => validator(val).err().map(|e| e.to_string()),
        Err(_) => Some("Invalid input".to_string())
    }
};

view! {
    <input
        on:input=move |ev| {
            let value = event_target_value(&ev);
            set_field_error.set(validate_field(&value, validate_fn));
        }
        class:error=move || field_error.get().is_some()
    />
}
```

### REST API Endpoints

Always maintain REST endpoints alongside server functions:

```rust
// In lib.rs router setup
let router = Router::new()
    .route("/api/plate", post(create_plate))  // REST API
    .leptos_routes(&leptos_options, routes, shell);  // Leptos routes

// Handler
async fn create_plate(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
    match validate(&payload) {
        Ok(_) => (StatusCode::CREATED, Json(response)),
        Err(e) => (StatusCode::BAD_REQUEST, Json(error))
    }
}
```

## Important Guidelines

### DO

- ✅ Use `ActionForm` for all forms
- ✅ Use `ServerAction` with `#[server]` functions
- ✅ Maintain both REST API and server functions
- ✅ Use shared validation logic from the validation crate
- ✅ Use `class:error` for conditional CSS classes
- ✅ Use reactive signals for UI state
- ✅ Keep validation crate `no_std` for WASM compatibility
- ✅ Provide individual field validators for real-time feedback
- ✅ Use `event_target_value(&ev)` to get input values
- ✅ Use `.get()` on signals in reactive contexts
- ✅ Use `.set()` to update signals

### DON'T

- ❌ Don't manually handle form submission with `on:submit` (use `ActionForm`)
- ❌ Don't use `type="button"` for submit buttons in `ActionForm`
- ❌ Don't make HTTP calls from client to `/api/*` endpoints when you can use server functions
- ❌ Don't remove or replace REST API endpoints - keep them for external consumers
- ❌ Don't use `std` features in the validation crate
- ❌ Don't duplicate validation logic - share it between client and server
- ❌ Don't use `on:submit:capture` - it causes hydration issues

## Feature Flags

The web crate uses conditional compilation:

- `ssr` - Server-side rendering features (Axum, Tokio, etc.)
- `hydrate` - Client-side hydration features (WASM)
- Default feature is `ssr`

## Build Commands

```bash
# Development with hot reload
cargo leptos watch

# Production build
cargo leptos build --release

# Server-side only (for testing)
cargo build -p web --features ssr

# Run tests
cargo test
```

## Validation Architecture

The validation crate provides:

1. **Full plate validation**: `validate(plate: &ActuatorPlate)`
2. **Individual field validators**:
   - `validate_bolt_spacing(value: u16)`
   - `validate_bolt_diameter(value: u16)`
   - `validate_bracket_height(value: u16)`
   - `validate_pin_diameter(value: u16)`
   - `validate_plate_thickness(value: u16)`

All validators return `Result<(), PlateValidationError>` and work in both WASM and native contexts.

## Progressive Enhancement

Forms should work without JavaScript:
- `ActionForm` handles this automatically
- Server functions degrade to POST requests
- Form field names must match server function parameters exactly

## Error Handling

- Use `ServerFnError::new(message)` for server function errors
- Display validation errors inline with fields
- Use the `pending()` method on actions for loading states
- Use the `value()` method on actions to access results

## Common Pitfalls

1. **Form not submitting**: Make sure you're using `ActionForm`, not manual form handling
2. **Hydration mismatches**: Ensure server and client render the same HTML
3. **Missing field validators**: Each form field should have real-time validation
4. **Type mismatches**: Form field names must exactly match server function parameter names
5. **Validation drift**: Keep client and server validation in sync by using the shared validation crate
6. **WASM import errors**: Never include server-only dependencies without feature gating them to `ssr` only. Server functions handle all client-server communication automatically.
7. **URL parameters on form submit**: If you see `?__path=...&__err=...` in the URL, it means WASM failed to load/hydrate. Check browser console for WASM import errors.

## Testing

We have comprehensive test coverage. See [TESTING.md](./TESTING.md) for details.

**Current test count: 18 tests**
- 13 validation unit tests (field validators + full plate + error messages)
- 5 REST API integration tests (health + valid/invalid requests)

### Running Tests

```bash
cargo test                           # All tests
cargo test -p validation            # Validation only
cargo test -p web --features ssr    # API integration tests
```

### Writing New Tests

When adding features:

1. **Validation logic**: Add unit tests in the validation crate
2. **API endpoints**: Add integration tests in `crates/web/tests/`
3. **Test both success and failure cases**
4. **Test error messages** to ensure they're user-friendly

Example validation test:
```rust
#[test]
fn test_validate_new_field_valid() {
    assert!(validate_new_field(10).is_ok());
}

#[test]
fn test_validate_new_field_invalid() {
    let result = validate_new_field(0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ValidationError::NewFieldInvalid));
}
```

## Questions to Ask

When implementing new features, consider:

1. Does this need both a REST API endpoint and a server function?
2. Should this validation run on both client and server?
3. Is the form handling idiomatic Leptos (using `ActionForm`)?
4. Does this maintain the API-first architecture?
5. Is the validation logic in the shared validation crate?
6. **Have I written tests for this feature?**
