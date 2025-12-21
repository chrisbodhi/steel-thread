# Frontend

React SPA for the actuator plate configurator.

## Development

```bash
bun install   # Install dependencies
bun dev       # Start dev server with HMR on http://localhost:3000
```

The dev server proxies `/api/*` requests to the Rust backend at `http://localhost:3030`.

## Build

```bash
bun run build
```

Outputs static files to `../crates/web/dist/` for the Rust server to serve in production.

## Tech Stack

- **Bun** - Runtime and bundler
- **React 19** - UI framework
- **TailwindCSS 4** - Styling
- **Radix UI** - Accessible component primitives
- **shadcn/ui pattern** - Component library approach

## Structure

```
src/
├── index.ts          # Bun server with API proxy
├── index.html        # HTML entry point
├── frontend.tsx      # React app root
├── App.tsx           # Main app component
├── components/ui/    # UI components (shadcn/ui style)
└── lib/utils.ts      # Utility functions
```

## Environment Variables

- `API_URL` - Backend API URL (default: `http://localhost:3030`)
