# Testing Guide

This project has comprehensive test coverage across multiple layers.

## Test Structure

```
crates/
├── validation/
│   └── src/lib.rs              # Unit tests for validation logic (15 tests)
├── parametric/
│   └── src/lib.rs              # Parametric generation tests (1 test)
└── web/
    └── tests/
        └── api_tests.rs        # Integration tests for REST API (5 tests)
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p validation
cargo test -p web --features ssr

# Run specific test
cargo test test_validate_bolt_spacing_valid

# Run with output
cargo test -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

## Test Coverage

### 1. Validation Logic Tests (`crates/validation/src/lib.rs`)

**Individual Field Validators** (12 tests):
- `test_validate_bolt_spacing_valid` - Valid values (60, 1, u16::MAX)
- `test_validate_bolt_spacing_invalid` - Zero value rejection
- `test_validate_bolt_diameter_valid` - Valid diameter
- `test_validate_bolt_diameter_invalid` - Zero diameter rejection
- `test_validate_bracket_height_valid` - Valid height
- `test_validate_bracket_height_invalid` - Zero height rejection
- `test_validate_bracket_width_valid` - Valid width
- `test_validate_bracket_width_invalid` - Zero width rejection
- `test_validate_pin_diameter_valid` - Valid pin diameter
- `test_validate_pin_diameter_invalid` - Zero pin diameter rejection
- `test_validate_plate_thickness_valid` - Valid thickness
- `test_validate_plate_thickness_invalid` - Zero thickness rejection

**Full Plate Validation** (2 tests):
- `test_validate_full_plate_valid` - Complete valid plate
- `test_validate_full_plate_invalid_bolt_spacing` - Plate with one invalid field

**Error Messages** (1 test):
- `test_error_display_messages` - Verify all error messages are correct

### 2. Parametric Tests (`crates/parametric/src/lib.rs`)

**Generation Tests** (1 test):
- `it_fails_when_it_should` - Validates that invalid plates fail generation

### 3. REST API Integration Tests (`crates/web/tests/api_tests.rs`)

**Endpoint Tests** (5 tests):
- `test_health_endpoint` - GET /api/health returns 200 OK
- `test_create_plate_valid` - POST /api/plate with valid data returns 201 CREATED
- `test_create_plate_invalid_bolt_spacing` - Invalid data returns 400 BAD_REQUEST
- `test_create_plate_invalid_json` - Malformed JSON returns 400 BAD_REQUEST
- `test_create_plate_all_fields_invalid` - All fields invalid returns 400 BAD_REQUEST

## Testing Patterns

### Validation Tests

Validation tests are pure unit tests with no dependencies:

```rust
#[test]
fn test_validate_bolt_spacing_valid() {
    assert!(validate_bolt_spacing(60).is_ok());
    assert!(validate_bolt_spacing(1).is_ok());
    assert!(validate_bolt_spacing(u16::MAX).is_ok());
}

#[test]
fn test_validate_bolt_spacing_invalid() {
    let result = validate_bolt_spacing(0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        PlateValidationError::BoltSpacingTooSmall
    ));
}
```

### API Integration Tests

API tests use Axum's testing utilities with `tower::ServiceExt::oneshot`:

```rust
#[tokio::test]
async fn test_create_plate_valid() {
    let app = create_test_router().await;

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(60),
        // ... other fields
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

## What's Not Tested (Yet)

### Server Functions
- The `#[server]` functions are harder to test in isolation
- They're implicitly tested through the web UI
- Could add end-to-end tests using browser automation

### Component Tests
- Leptos components are challenging to unit test
- Consider:
  - Browser-based testing (Playwright, Cypress)
  - Component testing with `wasm-bindgen-test`
  - Visual regression testing

### Future Test Ideas

1. **Property-based testing** with `proptest`:
   ```rust
   #[proptest]
   fn validate_never_panics(value: u16) {
       // Validation should never panic, only return errors
       let _ = validate_bolt_spacing(value);
   }
   ```

2. **Load testing** for the API:
   - Use `criterion` for benchmarks
   - Test concurrent requests

3. **Contract tests**:
   - Ensure API responses match expected schema
   - Test compatibility with external consumers

4. **Integration tests for full flow**:
   - Start the server
   - Make real HTTP requests
   - Verify database state (when we add persistence)

## Test Configuration

### Dev Dependencies (in `Cargo.toml`)

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
axum = { version = "0.8", features = ["json"] }
tokio = { version = "1.42", features = ["full"] }
http-body-util = "0.1"
serde_json = "1.0"
```

### Running Tests in CI/CD

```yaml
# Example GitHub Actions
- name: Run tests
  run: cargo test --workspace --verbose

- name: Run tests with features
  run: cargo test -p web --features ssr
```

## Best Practices

1. **Test both success and failure cases** - Every validator has valid/invalid tests
2. **Use descriptive test names** - Clear what's being tested
3. **Test error messages** - Ensure user-facing messages are correct
4. **Test edge cases** - Min/max values, empty inputs, etc.
5. **Keep tests fast** - Unit tests run in <1ms, integration tests in <10ms
6. **Test in isolation** - Each test creates its own router/data
7. **Use assertions that show helpful errors** - `assert_eq!` over `assert!`

## Continuous Testing

For active development, use:

```bash
# Watch mode - re-runs tests on file changes
cargo watch -x test

# Or with bacon
bacon test
```

## Coverage (Future)

To generate code coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html

# Open coverage/index.html
```

Current estimated coverage:
- **Validation crate**: ~100% (all functions tested)
- **Web crate**: ~40% (API endpoints tested, components not tested)
- **Domain crate**: 0% (simple types, no logic to test)
