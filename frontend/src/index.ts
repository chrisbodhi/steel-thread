import { serve } from "bun";
import index from "./index.html";

const API_URL = process.env.API_URL || "http://localhost:3030";
const POSTHOG_KEY = process.env.POSTHOG_KEY ?? "";
const POSTHOG_HOST = process.env.POSTHOG_HOST ?? "https://us.i.posthog.com";

const envScript = `<script>window.__ENV__={POSTHOG_KEY:${JSON.stringify(POSTHOG_KEY)},POSTHOG_HOST:${JSON.stringify(POSTHOG_HOST)}};</script>`;

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
      const fileName = url.pathname.split("/").pop();
      const file = Bun.file(`./src/wasm-validation/${fileName}`);

      if (await file.exists()) {
        const contentType = fileName?.endsWith(".wasm")
          ? "application/wasm"
          : fileName?.endsWith(".js")
            ? "application/javascript"
            : "text/plain";

        return new Response(file, {
          headers: {
            "Content-Type": contentType,
          },
        });
      }

      return new Response(
        "WASM module not found. Run 'just build-wasm' first.",
        { status: 404 },
      );
    },

    // Serve index.html for all other routes (SPA fallback)
    "/*": async () => {
      const html = await Bun.file(
        new URL("./index.html", import.meta.url),
      ).text();
      return new Response(html.replace("</head>", `${envScript}</head>`), {
        headers: { "Content-Type": "text/html" },
      });
    },
  },

  development: process.env.NODE_ENV !== "production" && {
    hmr: true,
    console: true,
  },
});

console.log(`🚀 Frontend running at ${server.url}`);
console.log(`📡 API proxy → ${API_URL}`);
