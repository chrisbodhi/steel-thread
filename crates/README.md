# Crates

This directory contains the Rust workspace crates for the actuator plate configuration application.

## Table of Contents

### [`domain/`](./domain)

Core domain types and models for the application.

- `ActuatorPlate` - Main domain model representing a plate configuration
- `Millimeters` - Type-safe measurement wrapper
- `no_std` compatible with Serde support

### [`validation/`](./validation)

Business logic for validating actuator plate configurations.

- Individual field validators (bolt spacing, diameter, height, etc.)
- Full plate validation
- `no_std` compatible
- Shared between backend and frontend

### [`web/`](./web)

Axum-based REST API server.

- HTTP endpoints (`/api/health`, `/api/generate`, `/api/download/step`, `/api/download/gltf`)
- Static file serving for the frontend
- Production binary target
- Integration with validation and parametric crates

### [`parametric/`](./parametric)

KCL parametric CAD definitions.

- `params.kcl` - Default plate parameters in KCL format
- Used for generating parametric 3D models

## Dependency Graph

```
web
├── domain
└── validation
    └── domain

parametric (independent)
```

## Building

```bash
# Build all crates
cargo build

# Build release
just build-release

# Test all crates
just test
```
