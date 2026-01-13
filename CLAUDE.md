# Claude Development Guide

This document provides context and guidelines for AI assistants (like Claude) working on this codebase.

## Project Overview

This is an actuator plate configuration application with:
- **Backend**: Axum-based REST API server (Rust)
- **Frontend**: React SPA with Bun (TypeScript)

In production, the Rust server serves both the API and the static frontend files from a single process.

## Technology Stack

### Backend (Rust)
- **Axum**: 0.8 (HTTP server)
- **Tokio**: 1.42 (async runtime)
- **Serde**: 1.0 (serialization)
- **tower-http**: Static file serving

### Frontend (Bun/React)
- **Bun**: Runtime and bundler
- **React**: 19
- **TailwindCSS**: 4.1
- **shadcn/ui**: Component library (new-york style)

## Architecture

### Directory Structure

```
├── crates/
│   ├── domain/       # Core domain types (ActuatorPlate, Millimeters)
│   ├── validation/   # no_std validation logic
│   └── web/          # Axum REST API server
│       └── dist/     # Built frontend assets (generated)
├── frontend/         # React SPA source
│   ├── src/
│   └── build.ts      # Bun build script
└── justfile          # Dev and build commands
```

### Architecture Decision: Single Server Deployment

We chose to serve the frontend as static files from the Rust server because:
- **Simpler deployment**: One binary/process to deploy
- **Lower cost**: No separate Node/Bun runtime in production
- **No CORS**: Frontend and API share the same origin

**Development**: Two servers (Bun for HMR, Rust for API)
**Production**: One server (Rust serves API + static files)

## Development Workflow

### Quick Start

```bash
# Install frontend dependencies (first time only)
cd frontend && bun install && cd ..

# Start both servers
just dev
```

This runs:
- Rust API on http://localhost:3030
- Bun frontend on http://localhost:3000 (with hot reload)

The Bun dev server proxies `/api/*` requests to the Rust backend.

### Development Tools

**Just** - Project orchestration and build pipeline
```bash
just dev            # Start both API and frontend servers
just dev-frontend   # Start only the frontend dev server
just build-release  # Build frontend + Rust for production
just test           # Run all tests once
just clean          # Clean build artifacts
```

**Bacon** - Interactive Rust development with file watching
```bash
bacon run-long   # Run API with auto-restart on changes
bacon test       # Run tests with auto-rerun on changes
bacon clippy     # Run clippy with auto-rerun on changes
bacon check      # Run type checking with auto-rerun
```

### Two-Terminal Workflow

For active backend development with frontend running:

```bash
# Terminal 1: Rust API with auto-reload
bacon run-long

# Terminal 2: Bun frontend with HMR
cd frontend && bun dev
```

## Build & Deploy

### Build for Production

```bash
just build-release
```

This:
1. Builds frontend to `crates/web/dist/`
2. Builds Rust binary with `--release`

### Run Production Build

```bash
./target/release/web
```

The server runs on port 3030 and serves:
- API endpoints at `/api/*`
- Frontend at all other routes

## REST API

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/generate` | Generate STEP and glTF model files |
| GET | `/api/download/step` | Download generated STEP file |
| GET | `/api/download/gltf` | Download generated glTF file |

## Frontend Development

### Key Files

- `frontend/src/index.ts` - Bun server with API proxy
- `frontend/src/index.html` - HTML entry point
- `frontend/src/frontend.tsx` - React app root
- `frontend/build.ts` - Production build script

### Adding Components

Use shadcn/ui for all UI components:

```bash
cd frontend
bunx shadcn@latest add <component-name>
```

Components are installed to `frontend/src/components/ui/`. You own the source code and can customize freely.

### Calling the API

```tsx
const response = await fetch('/api/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    bolt_spacing: 60,
    bolt_diameter: 10,
    bracket_height: 40,
    bracket_width: 30,
    pin_diameter: 10,
    pin_count: 6,
    plate_thickness: 8,
  }),
});
const data = await response.json();
```

## Validation Architecture

The validation crate provides:

1. **Full plate validation**: `validate(plate: &ActuatorPlate)`
2. **Individual field validators**:
   - `validate_bolt_spacing(value: u16)`
   - `validate_bolt_diameter(value: u16)`
   - `validate_bracket_height(value: u16)`
   - `validate_bracket_width(value: u16)`
   - `validate_pin_diameter(value: u16)`
   - `validate_plate_thickness(value: u16)`

All validators return `Result<(), PlateValidationError>`.

## Testing

**Current test count: 25 fast tests + 3 ignored integration tests**
- 18 validation unit tests
- 4 parametric unit tests
- 3 parametric integration tests (ignored - require zoo CLI)
- 3 REST API integration tests

```bash
just test                           # All fast tests (default)
cargo test -p validation            # Validation only
cargo test -p parametric            # Parametric tests (skips zoo CLI test)
cargo test -p parametric -- --ignored  # Run zoo CLI integration test
cargo test -p web                   # API tests only
```

See [TESTING.md](./TESTING.md) for detailed testing guide.

## Important Guidelines

### DO
- Use `just dev` for full-stack development
- Use `bunx shadcn@latest add <component>` to add new UI components
- Use shared validation logic from the validation crate
- Return detailed error messages in API responses
- Keep validation crate `no_std` compatible
- Build frontend before deploying (`just build`)

### DON'T
- Don't run Bun in production - serve static files from Rust
- Don't install Radix packages directly - use `bunx shadcn@latest add` instead
- Don't duplicate validation logic
- Don't use `std` features in the validation crate
- Don't forget to rebuild frontend after changes when testing production mode

## Questions to Ask

When implementing new features, consider:

1. Does this need a new API endpoint?
2. Is the validation logic in the shared validation crate?
3. Does the frontend need to be updated?
4. Have I written tests for this feature?
