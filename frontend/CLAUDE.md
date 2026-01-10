# Frontend Development Guide

This document provides Bun-specific guidance for working on the frontend.

## Project Context

This frontend is a **React SPA built with Bun**, served in two modes:

- **Development**: Bun dev server with HMR on http://localhost:3000 (proxies API to Rust backend)
- **Production**: Static files served by Rust backend at http://localhost:3030

**Backend is Rust (Axum), not Bun.** The Bun server is only used for development with hot module reloading.

## Bun Conventions

Use Bun instead of Node.js:

```bash
bun <file>                    # instead of node <file>
bun test                      # instead of jest/vitest
bun install                   # instead of npm/yarn/pnpm install
bun run <script>              # instead of npm run <script>
bunx <package>                # instead of npx <package>
```

Bun automatically loads `.env` files - no need for `dotenv`.

## Development Server

Our custom Bun dev server (`src/index.ts`) provides:
- Hot module reloading for React components
- API proxy: `/api/*` → `http://localhost:3030` (Rust backend)
- Static file serving in development

Start it with:
```bash
bun dev
```

## Frontend Tech Stack

- **React 19** - UI framework
- **TailwindCSS 4** - Styling  
- **shadcn/ui** - Component library (new-york style)
- **Lucide React** - Icons

## Adding UI Components

Use shadcn/ui for all components:

```bash
cd frontend
bunx shadcn@latest add <component-name>
```

This installs components to `src/components/ui/` as source code you own and can customize.

## HTML Imports Pattern

Our `index.html` imports React directly:

```html
<script type="module" src="./frontend.tsx"></script>
```

Bun's bundler automatically:
- Transpiles TypeScript/JSX
- Bundles dependencies
- Processes CSS imports (including Tailwind)
- Provides HMR in development

## Calling the Backend API

The frontend makes requests to `/api/*` which:
- In development: Proxied to `http://localhost:3030` by Bun dev server
- In production: Handled by Rust server (same origin)

Example:
```tsx
const response = await fetch('/api/plate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    bolt_spacing: 60,
    bolt_diameter: 10,
    bracket_height: 40,
    pin_diameter: 10,
    plate_thickness: 8,
  }),
});
const data = await response.json();
```

## Build for Production

```bash
bun run build
```

This runs `build.ts` which outputs static files to `../crates/web/dist/` for the Rust server to serve.

## Testing

Use `bun test` to run tests:

```ts
import { test, expect } from "bun:test";

test("example test", () => {
  expect(1).toBe(1);
});
```

## File Structure

```
src/
├── index.ts          # Bun dev server with API proxy
├── index.html        # HTML entry point
├── frontend.tsx      # React app root
├── App.tsx           # Main app component
├── components/ui/    # shadcn/ui components
└── lib/utils.ts      # Utility functions
```

## Important Notes

- **Don't use Bun APIs for backend work** - We use Rust (Axum) for the API server
- **Don't use `Bun.serve()` routes** - Our dev server uses a simple Express-like setup
- **Don't install Radix packages directly** - Use `bunx shadcn@latest add` instead
- **Build before testing production** - Run `just build` to update static assets

## Environment Variables

- `API_URL` - Backend API URL in development (default: `http://localhost:3030`)

## Full-Stack Development

Start both servers:
```bash
# From project root
just dev
```

Or in separate terminals:
```bash
# Terminal 1: Rust API with auto-reload
bacon run-long

# Terminal 2: Frontend with HMR
cd frontend && bun dev
```
