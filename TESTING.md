# Testing Guide

This project has comprehensive test coverage across multiple layers.

## Test Structure

```
crates/
├── validation/
│   └── src/lib.rs              # Unit tests for validation logic (18 tests)
├── parametric/
│   └── src/lib.rs              # Parametric generation tests (4 fast + 3 ignored)
└── web/
    └── tests/
        └── api_tests.rs        # Integration tests for REST API (6 tests)
```

**Total: 28 fast tests + 3 ignored integration tests**

## Running Tests

```bash
# Run all fast tests (default - skips ignored tests)
cargo test

# Run tests for a specific crate
cargo test -p validation      # Validation only (18 tests)
cargo test -p parametric      # Parametric tests (4 fast tests, skips 3 zoo CLI tests)
cargo test -p web             # API tests only (6 tests)

# Run specific test by name
cargo test test_validate_bolt_spacing_valid

# Run ignored integration tests (requires external dependencies)
cargo test -- --ignored

# Run ALL tests including ignored ones
cargo test -- --include-ignored

# Run with output
cargo test -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

### Running Specific Test Suites

```bash
# Run only validation tests
cargo test -p validation

# Run only parametric tests (skips zoo CLI integration test)
cargo test -p parametric

# Run parametric tests including the zoo CLI integration test
cargo test -p parametric -- --include-ignored

# Run only ignored tests (requires zoo CLI installed)
cargo test -p parametric -- --ignored

# Run only API tests
cargo test -p web
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

**Unit Tests** (4 tests):
- `test_generate_step_fails_with_invalid_plate` - Invalid plates fail validation
- `test_generate_model_succeeds_with_valid_plate` - Valid plates generate params file
- `test_generate_model_fails_with_invalid_plate` - Invalid plates return proper error
- `test_generate_params_file_creates_valid_kcl` - Generated KCL file has correct format and values

**Integration Tests** (1 ignored test):
- `test_generate_step_creates_file_with_zoo_cli` - Requires `zoo` CLI to be installed (marked `#[ignore]`)

The integration test is ignored by default because it requires:
1. The `zoo` CLI tool installed
2. `main.kcl` file to exist
3. `output_dir` directory to exist

Run with: `cargo test -p parametric -- --include-ignored`

### 3. REST API Integration Tests (`crates/web/tests/api_tests.rs`)

**Endpoint Tests** (3 tests):
- `test_health_endpoint` - GET /api/health returns 200 OK
- `test_generate_endpoint_invalid_plate` - POST /api/generate with invalid data returns 400 BAD_REQUEST
- `test_generate_endpoint_valid_plate` - POST /api/generate with valid data generates model files

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
async fn test_generate_endpoint_invalid_plate() {
    let app = create_test_router();

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(0), // Invalid!
        // ... other fields
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/generate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
2. **Use descriptive test names** - Clear what's being tested (e.g., `test_validate_bolt_spacing_valid`)
3. **Test error messages** - Ensure user-facing messages are correct
4. **Test edge cases** - Min/max values, empty inputs, etc.
5. **Keep tests fast** - Unit tests run in <1ms, integration tests in <10ms
6. **Test in isolation** - Each test creates its own router/data
7. **Use assertions that show helpful errors** - `assert_eq!` over `assert!`
8. **Mark external dependency tests with `#[ignore]`** - Tests requiring external tools (CLI, databases) should be ignored by default
9. **Clean up after yourself** - Tests that create files should remove them (use `.ok()` to ignore cleanup errors)
10. **Avoid silent failures** - Always check `Result` types explicitly with `assert!` or `unwrap()`

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
- **Parametric crate**: ~90% (all functions tested except external CLI integration)
- **Web crate**: ~40% (API endpoints tested, components not tested)
- **Domain crate**: 0% (simple types, no logic to test)

## Common Testing Anti-Patterns to Avoid

### ❌ Silent Failures
```rust
// BAD - test passes even if generate_step returns Err!
#[test]
fn bad_test() {
    if let Ok(status) = generate_step(plate) {
        assert!(status.success())
    }
}

// GOOD - explicitly check the result
#[test]
fn good_test() {
    let result = generate_step(plate);
    assert!(result.is_ok());
    assert!(result.unwrap().success());
}
```

### ❌ Tests That Don't Clean Up
```rust
// BAD - leaves files behind
#[test]
fn bad_test() {
    generate_params_file(&plate).unwrap();
    assert!(std::path::Path::new("params.kcl").exists());
    // params.kcl left behind!
}

// GOOD - cleans up after test
#[test]
fn good_test() {
    generate_params_file(&plate).unwrap();
    assert!(std::path::Path::new("params.kcl").exists());
    std::fs::remove_file("params.kcl").ok(); // cleanup
}
```

### ❌ External Dependencies Without #[ignore]
```rust
// BAD - breaks CI if CLI not installed
#[test]
fn bad_test() {
    let result = Command::new("zoo").status();
    assert!(result.is_ok());
}

// GOOD - marked as optional integration test
#[test]
#[ignore]
fn good_test() {
    let result = Command::new("zoo").status();
    assert!(result.is_ok());
}
```
