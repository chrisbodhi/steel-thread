# Development: run both servers
dev:
    #!/usr/bin/env bash
    echo "Starting API and frontend servers..."
    echo "API will run on http://localhost:3030"
    echo "Frontend will run on http://localhost:3000"
    trap 'kill 0' INT
    cargo run -p web &
    cd frontend && bun dev &
    wait

# Run just the Rust API server
dev-api:
    cargo run -p web

# Run just the Rust API server with bacon
dev-api-watch:
    bacon run-long

# Run just the Bun frontend dev server
dev-frontend:
    cd frontend && bun dev

# Build frontend for production
build:
    cd frontend && bun run build

# Build everything for production
build-release: build
    cargo build -p web --release

# Run all tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean
    rm -rf crates/web/dist
    rm -rf frontend/node_modules/.cache

# List available recipes
default:
    @just --list
