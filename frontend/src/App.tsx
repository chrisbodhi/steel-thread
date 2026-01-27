import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Button } from "./components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/ui/select";
import { ThemePicker } from "./components/ui/theme-picker";
import { AboutSection } from "./components/about-section";

import "./index.css";
import { useState, useEffect, useCallback, type FormEvent, type ChangeEvent } from "react";
import { ModelViewer } from "./components/model-viewer";
import {
  validateBoltSpacing,
  validateBoltSize,
  validateBracketHeight,
  validateBracketWidth,
  validateMaterial,
  validatePinDiameter,
  validatePinCount,
  validatePlateThickness,
  type ValidationResult,
} from "./lib/validation";

function Combined({
  forProp,
  name,
  defaultValue = "10",
  validator,
  onValidationChange,
  unit,
}: {
  forProp: string;
  name: string;
  defaultValue?: string;
  validator: (value: number) => Promise<ValidationResult>;
  onValidationChange?: (fieldName: string, isValid: boolean) => void;
  unit?: string;
}) {
  const [value, setValue] = useState(defaultValue);
  const [validationResult, setValidationResult] = useState<ValidationResult>({ valid: true });
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    const timeoutId = setTimeout(async () => {
      if (value && touched) {
        const result = await validator(Number(value));
        setValidationResult(result);
        onValidationChange?.(forProp, result.valid);
      }
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [value, validator, touched, forProp, onValidationChange]);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    setValue(e.target.value);
    if (!touched) {
      setTouched(true);
    }
  };

  const handleBlur = async () => {
    setTouched(true);
    if (value) {
      const result = await validator(Number(value));
      setValidationResult(result);
      onValidationChange?.(forProp, result.valid);
    }
  };

  const isInvalid = touched && !validationResult.valid;

  return (
    <div className="space-y-1.5">
      <Label htmlFor={forProp} className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {name}
        {unit && <span className="ml-1 text-[10px] opacity-60">({unit})</span>}
      </Label>
      <Input
        id={forProp}
        type="number"
        name={forProp}
        value={value}
        onChange={handleChange}
        onBlur={handleBlur}
        placeholder={defaultValue}
        className={isInvalid ? "border-destructive focus-visible:ring-destructive" : ""}
      />
      {isInvalid && (
        <p className="text-[10px] text-destructive font-medium">{validationResult.error}</p>
      )}
    </div>
  );
}

const BOLT_SIZES = ["M3", "M4", "M5", "M6", "M8", "M10", "M12"] as const;

const MATERIALS = [
  { value: "aluminum", label: "Aluminum 6061-T6" },
  { value: "stainless_steel", label: "Stainless Steel 304" },
  { value: "carbon_steel", label: "Carbon Steel" },
  { value: "brass", label: "Brass" },
] as const;

function BoltSizeSelect({
  forProp,
  name,
  defaultValue = "M10",
  onValidationChange,
}: {
  forProp: string;
  name: string;
  defaultValue?: string;
  onValidationChange?: (fieldName: string, isValid: boolean) => void;
}) {
  const [value, setValue] = useState(defaultValue);
  const [validationResult, setValidationResult] = useState<ValidationResult>({ valid: true });
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    const validateValue = async () => {
      if (value && touched) {
        const result = await validateBoltSize(value);
        setValidationResult(result);
        onValidationChange?.(forProp, result.valid);
      }
    };
    validateValue();
  }, [value, touched, forProp, onValidationChange]);

  const handleChange = (newValue: string) => {
    setValue(newValue);
    if (!touched) {
      setTouched(true);
    }
  };

  const isInvalid = touched && !validationResult.valid;

  return (
    <div className="space-y-1.5">
      <Label htmlFor={forProp} className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {name}
      </Label>
      <Select name={forProp} value={value} onValueChange={handleChange}>
        <SelectTrigger
          id={forProp}
          className={isInvalid ? "border-destructive focus-visible:ring-destructive" : ""}
        >
          <SelectValue placeholder="Select bolt size" />
        </SelectTrigger>
        <SelectContent>
          {BOLT_SIZES.map((size) => (
            <SelectItem key={size} value={size}>
              {size}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {isInvalid && (
        <p className="text-[10px] text-destructive font-medium">{validationResult.error}</p>
      )}
    </div>
  );
}

function MaterialSelect({
  forProp,
  name,
  defaultValue = "aluminum",
  onValidationChange,
}: {
  forProp: string;
  name: string;
  defaultValue?: string;
  onValidationChange?: (fieldName: string, isValid: boolean) => void;
}) {
  const [value, setValue] = useState(defaultValue);
  const [validationResult, setValidationResult] = useState<ValidationResult>({ valid: true });
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    const validateValue = async () => {
      if (value && touched) {
        const result = await validateMaterial(value);
        setValidationResult(result);
        onValidationChange?.(forProp, result.valid);
      }
    };
    validateValue();
  }, [value, touched, forProp, onValidationChange]);

  const handleChange = (newValue: string) => {
    setValue(newValue);
    if (!touched) {
      setTouched(true);
    }
  };

  const isInvalid = touched && !validationResult.valid;

  return (
    <div className="space-y-1.5">
      <Label htmlFor={forProp} className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {name}
      </Label>
      <Select name={forProp} value={value} onValueChange={handleChange}>
        <SelectTrigger
          id={forProp}
          className={isInvalid ? "border-destructive focus-visible:ring-destructive" : ""}
        >
          <SelectValue placeholder="Select material" />
        </SelectTrigger>
        <SelectContent>
          {MATERIALS.map((mat) => (
            <SelectItem key={mat.value} value={mat.value}>
              {mat.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {isInvalid && (
        <p className="text-[10px] text-destructive font-medium">{validationResult.error}</p>
      )}
    </div>
  );
}

function FieldGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <h3 className="text-[10px] font-bold uppercase tracking-[0.2em] text-primary/80 border-b border-primary/20 pb-1">
        {title}
      </h3>
      <div className="grid grid-cols-2 gap-3">
        {children}
      </div>
    </div>
  );
}

export function App() {
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [modelSrc, setModelSrc] = useState<string | null>(null);
  const [isPanelExpanded, setIsPanelExpanded] = useState(true);
  const [fieldValidationState, setFieldValidationState] = useState<Record<string, boolean>>({
    boltSpacing: true,
    boltSize: true,
    bracketHeight: true,
    bracketWidth: true,
    material: true,
    pinDiameter: true,
    pinCount: true,
    plateThickness: true,
  });

  const handleValidationChange = useCallback((fieldName: string, isValid: boolean) => {
    setFieldValidationState((prev) => ({
      ...prev,
      [fieldName]: isValid,
    }));
  }, []);

  const isFormValid = Object.values(fieldValidationState).every((isValid) => isValid);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    setIsLoading(true);
    setDownloadUrl(null);
    setErrorMessage(null);

    try {
      const form = e.currentTarget;
      const formData = new FormData(form);

      const body = JSON.stringify({
        bolt_spacing: Number(formData.get("boltSpacing")),
        bolt_size: formData.get("boltSize"),
        bracket_height: Number(formData.get("bracketHeight")),
        bracket_width: Number(formData.get("bracketWidth")),
        material: formData.get("material"),
        pin_diameter: Number(formData.get("pinDiameter")),
        pin_count: Number(formData.get("pinCount")),
        plate_thickness: Number(formData.get("plateThickness")),
      });

      const res = await fetch("/api/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body,
      });

      const data = await res.json();

      console.log(JSON.stringify(data, null, 2));

      if (data.success && data.download_url) {
        setDownloadUrl(data.download_url);
        setModelSrc(`${data.gltf_url}?t=${Date.now()}`);
      } else if (data.errors && data.errors.length > 0) {
        setErrorMessage(data.errors.join(", "));
      } else {
        setErrorMessage("An unknown error occurred");
      }
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen w-full flex flex-col relative">
      {/* Top navigation bar */}
      <header className="fixed top-0 left-0 right-0 z-50 px-4 py-3 lg:px-6 lg:py-4">
        <div className="flex items-center justify-between max-w-7xl mx-auto">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center backdrop-blur-sm">
              <svg className="w-6 h-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M12 3L3 8v8l9 5 9-5V8l-9-5z" />
                <path d="M12 12l9-5M12 12v9M12 12L3 8" />
              </svg>
            </div>
            <div>
              <h1 className="text-lg lg:text-xl font-bold tracking-tight">Platerator</h1>
              <p className="text-[10px] lg:text-xs text-muted-foreground uppercase tracking-wider">Actuator Configurator</p>
            </div>
          </div>
          <ThemePicker />
        </div>
      </header>

      {/* Main content area */}
      <main className="flex-1 flex flex-col lg:flex-row pt-20 lg:pt-24">
        {/* 3D Viewer - Hero section */}
        <div className="flex-1 relative min-h-[40vh] lg:min-h-0">
          <div className="absolute inset-4 lg:inset-8 rounded-2xl overflow-hidden border border-border/50 backdrop-blur-sm">
            {modelSrc ? (
              <ModelViewer src={modelSrc} alt="Actuator plate model" />
            ) : (
              <div className="w-full h-full flex flex-col items-center justify-center bg-muted/30 text-center p-8">
                <div className="w-20 h-20 lg:w-24 lg:h-24 rounded-2xl bg-primary/5 border border-primary/10 flex items-center justify-center mb-4 lg:mb-6">
                  <svg className="w-10 h-10 lg:w-12 lg:h-12 text-primary/40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                    <line x1="12" y1="22.08" x2="12" y2="12" />
                  </svg>
                </div>
                <p className="text-sm lg:text-base text-muted-foreground max-w-xs">
                  Configure your parameters and generate a model to preview it here
                </p>
              </div>
            )}
          </div>

          {/* Mobile panel toggle */}
          <button
            onClick={() => setIsPanelExpanded(!isPanelExpanded)}
            className="lg:hidden absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-1/2 z-20 px-6 py-2 rounded-full bg-primary text-primary-foreground text-sm font-medium shadow-lg flex items-center gap-2"
          >
            <span>{isPanelExpanded ? "Hide" : "Configure"}</span>
            <svg
              className={`w-4 h-4 transition-transform ${isPanelExpanded ? "rotate-180" : ""}`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>
        </div>

        {/* Configuration panel */}
        <div
          className={`
            lg:w-[420px] xl:w-[480px] shrink-0
            transition-all duration-300 ease-out
            ${isPanelExpanded ? "max-h-[70vh] lg:max-h-none" : "max-h-0 lg:max-h-none"}
            overflow-hidden lg:overflow-visible
          `}
        >
          <div className="h-full p-4 lg:p-6 lg:pr-8 space-y-4">
            <AboutSection />
            <Card className="h-full backdrop-blur-xl bg-card/80 border-border/50 shadow-2xl" data-card>
              <CardHeader className="pb-4">
                <CardTitle className="text-base lg:text-lg font-semibold flex items-center gap-2">
                  <svg className="w-5 h-5 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" />
                  </svg>
                  Configuration
                </CardTitle>
                <CardDescription className="text-xs">
                  Set your actuator plate specifications
                </CardDescription>
              </CardHeader>
              <CardContent className="overflow-y-auto max-h-[calc(70vh-180px)] lg:max-h-[calc(100vh-320px)]">
                <form onSubmit={handleSubmit} className="space-y-6">
                  <FieldGroup title="Dimensions">
                    <Combined
                      forProp="bracketHeight"
                      name="Height"
                      defaultValue="400"
                      validator={validateBracketHeight}
                      onValidationChange={handleValidationChange}
                      unit="mm"
                    />
                    <Combined
                      forProp="bracketWidth"
                      name="Width"
                      defaultValue="300"
                      validator={validateBracketWidth}
                      onValidationChange={handleValidationChange}
                      unit="mm"
                    />
                    <Combined
                      forProp="plateThickness"
                      name="Thickness"
                      defaultValue="8"
                      validator={validatePlateThickness}
                      onValidationChange={handleValidationChange}
                      unit="mm"
                    />
                    <MaterialSelect
                      forProp="material"
                      name="Material"
                      defaultValue="aluminum"
                      onValidationChange={handleValidationChange}
                    />
                  </FieldGroup>

                  <FieldGroup title="Fasteners">
                    <Combined
                      forProp="boltSpacing"
                      name="Bolt Spacing"
                      defaultValue="60"
                      validator={validateBoltSpacing}
                      onValidationChange={handleValidationChange}
                      unit="mm"
                    />
                    <BoltSizeSelect
                      forProp="boltSize"
                      name="Bolt Size"
                      defaultValue="M10"
                      onValidationChange={handleValidationChange}
                    />
                  </FieldGroup>

                  <FieldGroup title="Pins">
                    <Combined
                      forProp="pinDiameter"
                      name="Diameter"
                      defaultValue="10"
                      validator={validatePinDiameter}
                      onValidationChange={handleValidationChange}
                      unit="mm"
                    />
                    <Combined
                      forProp="pinCount"
                      name="Count"
                      defaultValue="6"
                      validator={validatePinCount}
                      onValidationChange={handleValidationChange}
                    />
                  </FieldGroup>

                  <div className="pt-2 space-y-3">
                    <Button
                      type="submit"
                      className="w-full h-11 text-sm font-semibold uppercase tracking-wider transition-all"
                      disabled={isLoading || !isFormValid}
                    >
                      {isLoading ? (
                        <span className="flex items-center gap-2">
                          <svg className="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
                            <path d="M12 2a10 10 0 0 1 10 10" />
                          </svg>
                          Generating...
                        </span>
                      ) : (
                        <span className="flex items-center gap-2">
                          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <polygon points="5 3 19 12 5 21 5 3" />
                          </svg>
                          Generate Model
                        </span>
                      )}
                    </Button>

                    {!isFormValid && (
                      <p className="text-[10px] text-muted-foreground text-center">
                        Fix validation errors to continue
                      </p>
                    )}

                    {downloadUrl && (
                      <div className="p-3 rounded-lg bg-primary/10 border border-primary/20">
                        <p className="text-xs text-primary font-medium mb-2 flex items-center gap-1.5">
                          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                            <polyline points="22 4 12 14.01 9 11.01" />
                          </svg>
                          Model generated successfully
                        </p>
                        <Button asChild variant="secondary" size="sm" className="w-full">
                          <a href={downloadUrl} download className="flex items-center gap-2">
                            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                              <polyline points="7 10 12 15 17 10" />
                              <line x1="12" y1="15" x2="12" y2="3" />
                            </svg>
                            Download STEP File
                          </a>
                        </Button>
                      </div>
                    )}

                    {errorMessage && (
                      <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20">
                        <p className="text-xs text-destructive font-medium flex items-center gap-1.5">
                          <svg className="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <circle cx="12" cy="12" r="10" />
                            <line x1="12" y1="8" x2="12" y2="12" />
                            <line x1="12" y1="16" x2="12.01" y2="16" />
                          </svg>
                          {errorMessage}
                        </p>
                      </div>
                    )}
                  </div>
                </form>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="py-4 px-6 text-center">
        <p className="text-[10px] text-muted-foreground/60 uppercase tracking-widest">
          Made in PGH / {new Date().getFullYear()} / AMDG
        </p>
      </footer>
    </div>
  );
}

export default App;
