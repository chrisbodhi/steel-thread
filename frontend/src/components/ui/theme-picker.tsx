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
    <div className="flex items-center gap-2">
      <Select
        value={colorScheme}
        onValueChange={(v) => setColorScheme(v as ColorScheme)}
      >
        <SelectTrigger className="w-[120px]">
          <Palette className="size-4" />
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {COLOR_SCHEMES.map((scheme) => (
            <SelectItem key={scheme.name} value={scheme.name}>
              {scheme.displayName}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <div className="flex items-center rounded-md border">
        {MODE_OPTIONS.map((option) => (
          <Button
            key={option.value}
            variant={mode === option.value ? "secondary" : "ghost"}
            size="icon-sm"
            onClick={() => setMode(option.value)}
            title={option.label}
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
