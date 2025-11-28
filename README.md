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
- **`crates/web`** - Web application with Leptos SSR and Axum API
  - Leptos SSR frontend for the plate configurator
  - Axum REST API endpoints (`/api/health`, `/api/plate`)
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

### API Endpoints

- `GET /api/health` - Health check endpoint
- `POST /api/plate` - Submit actuator plate configuration for validation

## Development

### Build everything
```bash
cargo build
```

### Build the web crate (server-side only)
```bash
cargo build -p web
```

Note: For full Leptos SSR build (server + WASM client), use `cargo leptos build`

### Run tests
```bash
cargo test
```

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

- [PLAN.md](./PLAN.md) - User flow outline and feature checklist
- [LEARNING.md](./LEARNING.md) - Success criteria and technical questions
- [Steel Thread for BAS](file://Users/b/Library/Mobile%20Documents/iCloud~md~obsidian/Documents/Algorithms/Steelthread%20for%20BAS.md) - Full context (Obsidian)
