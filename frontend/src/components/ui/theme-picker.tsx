import { Moon, Sun, Monitor, Palette } from "lucide-react";
import { Button } from "./button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./select";
import {
  useTheme,
  COLOR_SCHEMES,
  type ColorScheme,
  type ThemeMode,
} from "@/lib/theme";

const MODE_OPTIONS: { value: ThemeMode; label: string; icon: React.ReactNode }[] = [
  { value: "light", label: "Light", icon: <Sun className="size-4" /> },
  { value: "dark", label: "Dark", icon: <Moon className="size-4" /> },
  { value: "system", label: "System", icon: <Monitor className="size-4" /> },
];

export function ThemePicker() {
  const { colorScheme, mode, setColorScheme, setMode } = useTheme();

  return (
    <div className="flex items-center gap-1.5">
      <Select
        value={colorScheme}
        onValueChange={(v) => setColorScheme(v as ColorScheme)}
      >
        <SelectTrigger className="w-[110px] h-9 text-xs bg-background/60 border-border/40">
          <Palette className="size-3.5" />
          <SelectValue />
        </SelectTrigger>
        <SelectContent align="end">
          {COLOR_SCHEMES.map((scheme) => (
            <SelectItem key={scheme.name} value={scheme.name}>
              <span className="flex items-center gap-2">
                <span
                  className="size-2.5 rounded-full"
                  style={{
                    background: scheme.name === "cyber"
                      ? "linear-gradient(135deg, oklch(0.75 0.18 195), oklch(0.65 0.2 330))"
                      : scheme.name === "neutral"
                      ? "oklch(0.5 0 0)"
                      : scheme.name === "blue"
                      ? "oklch(0.6 0.15 250)"
                      : scheme.name === "green"
                      ? "oklch(0.6 0.15 145)"
                      : "oklch(0.6 0.15 350)",
                  }}
                />
                {scheme.displayName}
              </span>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <div className="flex items-center rounded-lg border border-border/40 bg-background/60 backdrop-blur-sm p-0.5">
        {MODE_OPTIONS.map((option) => (
          <Button
            key={option.value}
            variant={mode === option.value ? "secondary" : "ghost"}
            size="icon-sm"
            onClick={() => setMode(option.value)}
            title={option.label}
            className="size-8 rounded-md"
          >
            {option.icon}
          </Button>
        ))}
      </div>
    </div>
  );
}

export function ModeToggle() {
  const { mode, setMode, resolvedMode } = useTheme();

  const cycleMode = () => {
    const modes: ThemeMode[] = ["light", "dark", "system"];
    const currentIndex = modes.indexOf(mode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setMode(modes[nextIndex]!);
  };

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={cycleMode}
      title={`Current: ${mode}`}
    >
      {resolvedMode === "dark" ? (
        <Moon className="size-4" />
      ) : (
        <Sun className="size-4" />
      )}
    </Button>
  );
}
