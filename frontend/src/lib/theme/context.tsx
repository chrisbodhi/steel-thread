import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
} from "react";
import type { ColorScheme, ThemeContextValue, ThemeMode } from "./types";
import { DEFAULT_COLOR_SCHEME, DEFAULT_MODE, STORAGE_KEYS } from "./config";

const ThemeContext = createContext<ThemeContextValue | null>(null);

function useSystemPreference(): "light" | "dark" {
  const subscribe = useCallback((callback: () => void) => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", callback);
    return () => mediaQuery.removeEventListener("change", callback);
  }, []);

  const getSnapshot = useCallback(() => {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }, []);

  const getServerSnapshot = useCallback(() => "light" as const, []);

  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}

interface ThemeProviderProps {
  children: React.ReactNode;
  defaultColorScheme?: ColorScheme;
  defaultMode?: ThemeMode;
}

export function ThemeProvider({
  children,
  defaultColorScheme = DEFAULT_COLOR_SCHEME,
  defaultMode = DEFAULT_MODE,
}: ThemeProviderProps) {
  const [colorScheme, setColorSchemeState] = useState<ColorScheme>(() => {
    if (typeof window === "undefined") return defaultColorScheme;
    const stored = localStorage.getItem(STORAGE_KEYS.colorScheme);
    return (stored as ColorScheme) || defaultColorScheme;
  });

  const [mode, setModeState] = useState<ThemeMode>(() => {
    if (typeof window === "undefined") return defaultMode;
    const stored = localStorage.getItem(STORAGE_KEYS.mode);
    return (stored as ThemeMode) || defaultMode;
  });

  const systemPreference = useSystemPreference();
  const resolvedMode = mode === "system" ? systemPreference : mode;

  useEffect(() => {
    const root = document.documentElement;

    root.dataset.theme = colorScheme;

    if (resolvedMode === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }, [colorScheme, resolvedMode]);

  const setColorScheme = useCallback((scheme: ColorScheme) => {
    setColorSchemeState(scheme);
    localStorage.setItem(STORAGE_KEYS.colorScheme, scheme);
  }, []);

  const setMode = useCallback((newMode: ThemeMode) => {
    setModeState(newMode);
    localStorage.setItem(STORAGE_KEYS.mode, newMode);
  }, []);

  const value = useMemo<ThemeContextValue>(
    () => ({
      colorScheme,
      mode,
      resolvedMode,
      setColorScheme,
      setMode,
    }),
    [colorScheme, mode, resolvedMode, setColorScheme, setMode]
  );

  return (
    <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}
