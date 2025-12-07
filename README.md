# Steel Thread for BAS

An actuator plate configurator exploring web → parametric CAD → quote pipelines.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (JavaScript runtime)
- [Just](https://github.com/casey/just) (command runner) - install via `cargo install just` or `brew install just`

## Quick Start

```bash
# Install frontend dependencies
cd frontend && bun install && cd ..

# Start development servers
just dev
```

This starts:
- **API**: http://localhost:3030 (Rust/Axum)
- **Frontend**: http://localhost:3000 (Bun/React with HMR)

## Project Structure

```
├── crates/
│   ├── domain/       # Core types (ActuatorPlate, Millimeters)
│   ├── validation/   # Business logic (no_std, field validators)
│   └── web/          # Axum REST API server
├── frontend/         # React SPA (Bun, TailwindCSS, Radix UI)
└── justfile          # Dev and build commands
This is a Rust workspace with multiple crates:

- **`crates/domain`** - Core domain types and data structures
  - `ActuatorPlate` - Main domain model
  - `Millimeters` - Type-safe unit wrapper
  - Shared across validation and web layers
- **`crates/parametric`** - Interface for creating STEP files
  - Accepts a Validation trait (TFTF -- truths from the future)
  - Creates a STEP file of the desired object
  - Does it also create a 3D model or rendering? Or should that happen elsewhere?
- **`crates/validation`** - Business logic and validation rules
  - Manufacturing constraint checks
  - Geometric validation
  - `no_std` compatible for WASM compilation
  - Individual field validators for real-time client-side validation
  - Shared validation logic across client and server
  - Depends on domain types
- **`crates/web`** - Web application with Axum API
  - Server functions for type-safe client-server communication
  - Real-time field validation using validation crate compiled to WASM
  - Depends on domain and validation crates

## Running the Project

### Prerequisites

## Architecture

**Development**: Two servers - Bun (frontend with HMR) + Rust (API)

**Production**: One server - Rust serves API + static frontend

```
┌─────────────────────────────────────────┐
│            Production Server            │
│                                         │
│   /api/*  →  Axum handlers              │
│   /*      →  Static files (React SPA)   │
│                                         │
└─────────────────────────────────────────┘
```

## Development Commands

### Just (Project Orchestration)

```bash
just dev            # Start both servers for development
just dev-frontend   # Start only frontend dev server
just build-release  # Build frontend + Rust for production
just test           # Run all tests once
just clean          # Clean build artifacts
just                # List all available commands
```

### Bacon (Interactive Rust Development)

```bash
bacon run-long   # Run API with auto-restart on file changes
bacon test       # Run tests with auto-rerun on changes
bacon clippy     # Run linting with auto-rerun on changes
bacon check      # Run type checking with auto-rerun
```

### Workflow Patterns

**Full-stack development:**
```bash
just dev
```

**Backend-focused development (two terminals):**
```bash
# Terminal 1
bacon run-long

# Terminal 2
cd frontend && bun dev
```

**Frontend-only:**
```bash
just dev-frontend
# or
cd frontend && bun dev
```

**Quick test run:**
```bash
just test
```

**Test-driven development:**
```bash
bacon test
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/plate` | Validate plate configuration |

## Documentation

- [CLAUDE.md](./CLAUDE.md) - Development guide (architecture, patterns, workflows)
- [TESTING.md](./TESTING.md) - Testing documentation
- [PLAN.md](./PLAN.md) - Feature roadmap
- [LEARNING.md](./LEARNING.md) - Technical exploration notes
