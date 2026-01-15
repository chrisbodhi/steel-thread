# Bracket Racket

An actuator plate configurator exploring web → parametric CAD → quote pipelines.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (JavaScript runtime)
- [Just](https://github.com/casey/just) (command runner) - install via `cargo install just` or `brew install just`
- [Terraform](https://...) (IaC)
- [AWS CLI](https://...) (Cloud provider)

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
├── crates/           # Rust workspace (see crates/README.md)
│   ├── domain/       # Core types (ActuatorPlate, Millimeters)
│   ├── validation/   # Business logic (no_std, field validators)
│   ├── web/          # Axum REST API server
│   └── parametric/   # KCL parametric CAD definitions
├── frontend/         # React SPA (Bun, TailwindCSS, shadcn/ui)
└── justfile          # Dev and build commands
```

See [crates/README.md](./crates/README.md) for detailed crate documentation.

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

### Deploying

After getting `terraform` and the `aws` CLI installed, make sure you've logged in with `aws login`.

Run `terraform apply` to deploy your changes. Run `terraform state list` and `terraform show` for more details.



## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/generate` | Generate STEP and glTF model files |
| GET | `/api/download/step` | Download generated STEP file |
| GET | `/api/download/gltf` | Download generated glTF file |

## Documentation

- [CLAUDE.md](./CLAUDE.md) - Development guide (architecture, patterns, workflows)
- [crates/README.md](./crates/README.md) - Rust crates documentation
- [frontend/README.md](./frontend/README.md) - Frontend documentation
- [TESTING.md](./TESTING.md) - Testing documentation
- [PLAN.md](./PLAN.md) - Feature roadmap
- [LEARNING.md](./LEARNING.md) - Technical exploration notes
