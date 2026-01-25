# Claude Development Guide

This document provides context and guidelines for AI assistants (like Claude) working on this codebase.

## Project Overview

**Platerator** is an actuator plate configuration application with:
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
- **WebAssembly**: Shared validation logic compiled from Rust

### Prerequisites

- **Rust**: Latest stable
- **Bun**: 1.0+ (for frontend development)
- **wasm-pack**: 0.14.0+ (for building WASM modules)
  ```bash
  cargo install wasm-pack
  ```
- **Just**: Command runner (optional but recommended)
  ```bash
  cargo install just
  ```

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
just dev            # Start both API and frontend servers (builds WASM first)
just dev-frontend   # Start only the frontend dev server (builds WASM first)
just build-wasm     # Build WASM validation module for frontend
just build-release  # Build frontend + Rust for production
just test           # Run all tests once
just clean          # Clean build artifacts (including WASM)
just stop           # Stop any running dev servers
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

### Troubleshooting

**Port Already in Use Error**

If you see "Address already in use" when running `just dev`:

```bash
# Stop any running dev servers
just stop

# Or manually check what's using the ports
lsof -i :3030  # API server port
lsof -i :3000  # Frontend dev server port

# Then start again
just dev
```

**WASM Build Issues**

If wasm-pack fails to build:

1. Ensure you have wasm-pack 0.14.0+:
   ```bash
   wasm-pack --version
   cargo install wasm-pack --force  # Update if needed
   ```

2. The WASM module is built automatically when running `just dev` or `just build`

3. To manually rebuild just the WASM module:
   ```bash
   just build-wasm
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

## CI/CD

### GitHub Actions Workflows

The project uses GitHub Actions for continuous integration and deployment:

**CI Workflow** (`.github/workflows/ci.yml`):
- Runs on all pull requests targeting master
- **test-rust**: Runs `cargo test` and `cargo clippy`
- **test-frontend**: Builds frontend to catch TypeScript errors

**Build Workflow** (`.github/workflows/build.yml`):
- Runs on pushes to master
- **test**: Runs all tests and clippy
- **build**: Builds the release binary for deployment to Lightsail

### IMPORTANT: Cache Busting When Adding Dependencies

**When adding new Rust crates**, you must bump the cache version in **BOTH** GitHub Actions workflows to prevent cache incompatibility issues.

#### Why This Matters

GitHub Actions caches the `target/` directory and cargo registry to speed up builds. When you add new dependencies (especially large ones like `utoipa`, AWS SDK crates, etc.), the cached build artifacts may become incompatible, causing mysterious build failures in CI even though the code compiles locally.

#### How to Bump the Cache

Update the cache version prefix in **ALL** of these locations:

**1. In `.github/workflows/ci.yml`** (PR checks):
```yaml
# Change this:
key: ${{ runner.os }}-cargo-v2-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-cargo-v2-

# To this (increment the version number):
key: ${{ runner.os }}-cargo-v3-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-cargo-v3-
```

Update in:
- The test-rust job cache (~line 20)

**2. In `.github/workflows/build.yml`** (master builds):

Update in:
- The test job cache (~line 20)
- The build job cache (~line 49)

#### When to Bump

Bump the cache version (v2 → v3 → v4, etc.) when:
- Adding new crate dependencies to any `Cargo.toml`
- Upgrading major versions of existing dependencies
- CI build fails but local build succeeds
- You see "error: failed to select a version" or similar dependency resolution errors in CI

**CRITICAL**: Keep cache versions synchronized across both workflows to avoid confusion.

#### Example

See commit `8db1195` for a real example of fixing a cache incompatibility issue after adding utoipa dependencies.

## REST API

### OpenAPI Documentation

The API is fully documented with OpenAPI 3.0 (formerly Swagger):

- **Swagger UI**: http://localhost:3030/api/docs (interactive documentation)
- **OpenAPI spec**: http://localhost:3030/api/openapi.json (machine-readable)

Documentation is generated using `utoipa` and `utoipa-swagger-ui` crates.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/validate` | Validate plate parameters without generating files |
| POST | `/api/generate` | Generate STEP and glTF model files |
| GET | `/api/download/step/{session_id}` | Download generated STEP file |
| GET | `/api/download/gltf/{session_id}` | Download generated glTF file |
| GET | `/api/docs` | Interactive Swagger UI documentation |
| GET | `/api/openapi.json` | OpenAPI specification (JSON) |

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
    bolt_size: "M10",  // Standard ISO metric bolt size (M3, M4, M5, M6, M8, M10, M12)
    bracket_height: 40,
    bracket_width: 30,
    material: "aluminum",  // Material: aluminum, stainless_steel, carbon_steel, or brass
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
   - `validate_bolt_size(value: &str)` - Validates standard ISO metric sizes (M3, M4, M5, M6, M8, M10, M12)
   - `validate_bracket_height(value: u16)`
   - `validate_bracket_width(value: u16)`
   - `validate_material(value: &str)` - Validates material types (aluminum, stainless_steel, carbon_steel, brass)
   - `validate_pin_diameter(value: u16)`
   - `validate_pin_count(value: u16)` - Validates count is between 1 and 12
   - `validate_plate_thickness(value: u16)`

All validators return `Result<(), PlateValidationError>`.

## Testing

**Current test count: 35 fast tests + 3 ignored integration tests**
- 20 validation unit tests
- 4 parametric unit tests
- 3 parametric integration tests (ignored - require zoo CLI)
- 5 web crate unit tests
- 6 REST API integration tests

```bash
just test                           # All fast tests (default)
cargo test -p validation            # Validation only
cargo test -p parametric            # Parametric tests (skips zoo CLI test)
cargo test -p parametric -- --ignored  # Run zoo CLI integration test
cargo test -p web                   # API tests only
```

See [TESTING.md](./TESTING.md) for detailed testing guide.

## Adding New API Endpoints

When adding new API endpoints, follow these steps:

1. **Define request/response types** with `ToSchema` derive:
   ```rust
   use utoipa::ToSchema;

   #[derive(Serialize, Deserialize, ToSchema)]
   struct MyRequest {
       /// Field description (appears in OpenAPI docs)
       field: String,
   }
   ```

2. **Add endpoint handler** with `#[utoipa::path]` annotation:
   ```rust
   #[utoipa::path(
       post,
       path = "/api/my-endpoint",
       tag = "my-tag",
       request_body = MyRequest,
       responses(
           (status = 200, description = "Success", body = MyResponse),
           (status = 400, description = "Invalid request", body = ErrorResponse)
       )
   )]
   async fn my_handler(Json(payload): Json<MyRequest>) -> impl IntoResponse {
       // Implementation
   }
   ```

3. **Register in OpenAPI doc** (`ApiDoc` struct):
   - Add to `paths()` list
   - Add new schemas to `components(schemas())`
   - Add new tag to `tags()` if needed

4. **Add route** in `create_router()` function

The OpenAPI documentation will automatically update and be visible at `/api/docs`.

## Important Guidelines

### DO
- Use `just dev` for full-stack development
- Use `bunx shadcn@latest add <component>` to add new UI components
- Use shared validation logic from the validation crate
- Return detailed error messages in API responses
- Keep validation crate `no_std` compatible
- Build frontend before deploying (`just build`)
- Add OpenAPI documentation (`#[utoipa::path]`) to all new API endpoints
- Document all API request/response types with `ToSchema` derive
- **Bump GitHub Actions cache version** when adding new Rust dependencies (see CI/CD section)

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
5. **If adding Rust dependencies**: Did I bump the GitHub Actions cache version in **BOTH** `.github/workflows/ci.yml` and `.github/workflows/build.yml`?
