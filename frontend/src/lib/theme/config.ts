import type { ColorScheme, ThemeMode } from "./types";

export interface ColorSchemeInfo {
  name: ColorScheme;
  displayName: string;
}

export const COLOR_SCHEMES: ColorSchemeInfo[] = [
  { name: "neutral", displayName: "Neutral" },
  { name: "blue", displayName: "Blue" },
  { name: "green", displayName: "Green" },
  { name: "rose", displayName: "Rose" },
];

export const DEFAULT_COLOR_SCHEME: ColorScheme = "neutral";
export const DEFAULT_MODE: ThemeMode = "system";

export const STORAGE_KEYS = {
  colorScheme: "theme-color-scheme",
  mode: "theme-mode",
} as const;
