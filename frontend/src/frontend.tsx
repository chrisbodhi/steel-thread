/**
 * This file is the entry point for the React app, it sets up the root
 * element and renders the App component to the DOM.
 *
 * It is included in `src/index.html`.
 */

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { PostHogProvider } from "@posthog/react";
import { ThemeProvider } from "@/lib/theme";
import { App } from "./App";

declare const window: Window & {
  __ENV__?: { POSTHOG_KEY: string; POSTHOG_HOST: string };
};
const POSTHOG_KEY = window.__ENV__?.POSTHOG_KEY ?? "";
const POSTHOG_HOST = window.__ENV__?.POSTHOG_HOST ?? "https://us.i.posthog.com";

const elem = document.getElementById("root")!;
const app = (
  <StrictMode>
    <PostHogProvider apiKey={POSTHOG_KEY} options={{ api_host: POSTHOG_HOST }}>
      <ThemeProvider>
        <App />
      </ThemeProvider>
    </PostHogProvider>
  </StrictMode>
);

if (import.meta.hot) {
  // With hot module reloading, `import.meta.hot.data` is persisted.
  const root = (import.meta.hot.data.root ??= createRoot(elem));
  root.render(app);
} else {
  // The hot module reloading API is not available in production.
  createRoot(elem).render(app);
}
