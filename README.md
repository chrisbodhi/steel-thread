# Steel Thread for BAS

An actuator plate configurator exploring web → parametric CAD → quote pipelines.

## Project Structure

This is a Rust workspace with multiple crates:

- **`crates/domain`** - Core domain types and data structures
  - `ActuatorPlate` - Main domain model
  - `Millimeters` - Type-safe unit wrapper
  - Shared across validation and web layers
- **`crates/validation`** - Business logic and validation rules
  - Manufacturing constraint checks
  - Geometric validation
  - Depends on domain types
- **`crates/web`** - Web server (Axum) for the plate configurator
  - Standalone deployable binary
  - Handles form submissions and plate orders
  - Serves static HTML interface
  - Depends on domain and validation crates

## Running the Project

### Run the web server:

```bash
# Run from workspace root
cargo run -p web

# Or build and run the binary
cargo build -p web --release
./target/release/web
```

The server will start on `http://localhost:3030`

## Development

### Build everything
```bash
cargo build
```

### Build just the web crate
```bash
cargo build -p web
```

### Run tests
```bash
cargo test
```

### With `bacon`

#### Run the web endpoint

```bash
bacon run-long
```

## Project Documentation

- [PLAN.md](./PLAN.md) - User flow outline and feature checklist
- [LEARNING.md](./LEARNING.md) - Success criteria and technical questions
- [Steel Thread for BAS](file://Users/b/Library/Mobile%20Documents/iCloud~md~obsidian/Documents/Algorithms/Steelthread%20for%20BAS.md) - Full context (Obsidian)
