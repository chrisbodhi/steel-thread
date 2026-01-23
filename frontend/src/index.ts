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

    // Serve WASM validation module files
    "/wasm-validation/*": async (req) => {
      const url = new URL(req.url);
      const fileName = url.pathname.split('/').pop();
      const file = Bun.file(`./src/wasm-validation/${fileName}`);

      if (await file.exists()) {
        const contentType = fileName?.endsWith('.wasm')
          ? 'application/wasm'
          : fileName?.endsWith('.js')
          ? 'application/javascript'
          : 'text/plain';

        return new Response(file, {
          headers: {
            "Content-Type": contentType,
          },
        });
      }

      return new Response("WASM module not found. Run 'just build-wasm' first.", { status: 404 });
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
