# Frontend

React SPA for **Platerator**, an actuator plate configurator.

See the [root README](../README.md) for development workflow and commands.

For Bun-specific guidance, see [CLAUDE.md](./CLAUDE.md).

## Quick Start

```bash
# From project root
just dev            # Start both API and frontend
just dev-frontend   # Start only frontend

# Or from this directory
bun install         # Install dependencies (first time)
bun dev             # Start dev server with HMR
bun run build       # Build for production
```

## Tech Stack

- **React 19** - UI framework
- **TailwindCSS 4** - Styling
- **shadcn/ui** - Component library (new-york style)
- **Lucide React** - Icons

## Adding Components

```bash
bunx shadcn@latest add <component-name>
```

Components install to `src/components/ui/` as source code you own.
