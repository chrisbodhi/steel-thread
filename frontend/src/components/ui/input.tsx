import * as React from "react";

import { cn } from "@/lib/utils";

function Input({ className, type, ...props }: React.ComponentProps<"input">) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        // Base styles
        "h-10 w-full min-w-0 rounded-lg border bg-background/50 px-3 py-2 text-sm shadow-sm backdrop-blur-sm",
        // Border and colors
        "border-border/60 placeholder:text-muted-foreground/60",
        // Selection
        "selection:bg-primary selection:text-primary-foreground",
        // Transitions
        "transition-all duration-200 outline-none",
        // Focus state with glow
        "focus-visible:border-primary/50 focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:bg-background/80",
        // Dark mode enhancements
        "dark:bg-input/20 dark:border-border/40 dark:focus-visible:bg-input/30",
        // Invalid state
        "aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/30",
        // File input
        "file:text-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium",
        // Disabled
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        // Number input - hide spinners for cleaner look
        "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none",
        className,
      )}
      {...props}
    />
  );
}

export { Input };
