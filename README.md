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
  - `no_std` compatible for WASM compilation
  - Individual field validators for real-time client-side validation
  - Shared validation logic across client and server
  - Depends on domain types
- **`crates/web`** - Web application with Leptos SSR and Axum API
  - Leptos SSR frontend with reactive form validation
  - Server functions for type-safe client-server communication
  - Real-time field validation using validation crate compiled to WASM
  - Server-side rendering with client-side hydration
  - Depends on domain and validation crates

## Running the Project

### Prerequisites

Install cargo-leptos for building and running the SSR application:

```bash
cargo install cargo-leptos
```

### Run the web application:

```bash
# Run in development mode with hot reload
cargo leptos watch

# Or build and run for production
cargo leptos build --release
cargo leptos serve --release
```

The server will start on `http://localhost:3030`

### Features

- **Reactive Form Validation** - Real-time field validation as users type
- **Server Functions** - Type-safe RPC between client and server using `#[server]` macro
- **Progressive Enhancement** - Form works with and without JavaScript
- **Shared Validation Logic** - Same validation code runs on both client (WASM) and server
- **SSR with Hydration** - Fast initial page load with server-side rendering

### API Endpoints

- `GET /api/health` - Health check endpoint
- Server functions automatically registered by Leptos (e.g., `submit_plate`)

## Development

### Build everything
```bash
cargo build
```

### Build the web crate
```bash
# Server-side only (faster for testing server code)
cargo build -p web --features ssr

# Full SSR build (server + WASM client)
cargo leptos build
```

Note: The validation crate is automatically compiled to WASM for client-side validation

### Run tests
```bash
# Run all tests (18 tests across validation + API)
cargo test

# Run tests for specific crate
cargo test -p validation
cargo test -p web --features ssr
```

See [TESTING.md](./TESTING.md) for detailed testing documentation.

### With `bacon`

Install with

```bash
cargo install --locked bacon --features "clipboard sound"
```

#### Run the web endpoint

```bash
bacon run-long
```

## Project Documentation

- [TESTING.md](./TESTING.md) - Comprehensive testing guide (18 tests: validation + API)
- [CLAUDE.md](./CLAUDE.md) - AI assistant development guide (architecture, patterns, versions)
- [PLAN.md](./PLAN.md) - User flow outline and feature checklist
- [LEARNING.md](./LEARNING.md) - Success criteria and technical questions
- [Steel Thread for BAS](file://Users/b/Library/Mobile%20Documents/iCloud~md~obsidian/Documents/Algorithms/Steelthread%20for%20BAS.md) - Full context (Obsidian)
