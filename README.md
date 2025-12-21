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
```

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

## Commands

```bash
just dev            # Start both servers for development
just build-release  # Build frontend + Rust for production
just test           # Run all tests
just                # List all available commands
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/plate` | Validate plate configuration |

## Development

### Using bacon (Rust file watcher)

```bash
bacon run-long   # API with auto-restart
bacon test       # Tests with auto-rerun
bacon clippy     # Linting with auto-rerun
```

### Frontend only

```bash
cd frontend && bun dev
```

### Running tests

```bash
cargo test                    # All tests (18 total)
cargo test -p validation      # Validation tests (13)
cargo test -p web             # API tests (5)
```

## Documentation

- [CLAUDE.md](./CLAUDE.md) - Development guide (architecture, patterns, workflows)
- [TESTING.md](./TESTING.md) - Testing documentation
- [PLAN.md](./PLAN.md) - Feature roadmap
- [LEARNING.md](./LEARNING.md) - Technical exploration notes
