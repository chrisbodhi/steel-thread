export type ColorScheme = "neutral" | "blue" | "green" | "rose";
export type ThemeMode = "light" | "dark" | "system";

export interface ThemeContextValue {
  colorScheme: ColorScheme;
  mode: ThemeMode;
  resolvedMode: "light" | "dark";
  setColorScheme: (scheme: ColorScheme) => void;
  setMode: (mode: ThemeMode) => void;
}
