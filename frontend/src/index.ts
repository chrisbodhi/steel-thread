import { serve } from "bun";
import index from "./index.html";

const API_URL = process.env.API_URL || "http://localhost:3030";

const server = serve({
  routes: {
    // Proxy all /api/* requests to Rust backend
    "/api/*": async (req) => {
      const url = new URL(req.url);
      return fetch(`${API_URL}${url.pathname}`, {
        method: req.method,
        headers: req.headers,
        body: req.body,
      });
    },

    // Serve index.html for all other routes (SPA fallback)
    "/*": index,
  },

  development: process.env.NODE_ENV !== "production" && {
    hmr: true,
    console: true,
  },
});

console.log(`🚀 Frontend running at ${server.url}`);
console.log(`📡 API proxy → ${API_URL}`);
