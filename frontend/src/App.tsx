import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Button } from "./components/ui/button";
import { ThemePicker } from "./components/ui/theme-picker";

import "./index.css";
import { useState, useEffect, type FormEvent, type ChangeEvent } from "react";
import { ModelViewer } from "./components/model-viewer";
import {
  validateBoltSpacing,
  validateBoltDiameter,
  validateBracketHeight,
  validateBracketWidth,
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
}: {
  forProp: string;
  name: string;
  defaultValue?: string;
  validator: (value: number) => Promise<ValidationResult>;
  onValidationChange?: (fieldName: string, isValid: boolean) => void;
}) {
  const [value, setValue] = useState(defaultValue);
  const [validationResult, setValidationResult] = useState<ValidationResult>({ valid: true });
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    // Validate on value change with debounce
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

  return (
    <div>
      <Label htmlFor={forProp}>{name}</Label>
      <Input
        id={forProp}
        type="number"
        name={forProp}
        value={value}
        onChange={handleChange}
        onBlur={handleBlur}
        placeholder={defaultValue}
        className={
          touched && !validationResult.valid
            ? "border-red-500 focus-visible:ring-red-500"
            : ""
        }
      />
      {touched && !validationResult.valid && (
        <p className="text-xs text-red-500 mt-1">{validationResult.error}</p>
      )}
    </div>
  );
}

export function App() {
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [modelSrc, setModelSrc] = useState<string | null>(null);
  const [fieldValidationState, setFieldValidationState] = useState<Record<string, boolean>>({
    boltSpacing: true,
    boltDiameter: true,
    bracketHeight: true,
    bracketWidth: true,
    pinDiameter: true,
    pinCount: true,
    plateThickness: true,
  });

  const handleValidationChange = (fieldName: string, isValid: boolean) => {
    setFieldValidationState((prev) => ({
      ...prev,
      [fieldName]: isValid,
    }));
  };

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
        bolt_diameter: Number(formData.get("boltDiameter")),
        bracket_height: Number(formData.get("bracketHeight")),
        bracket_width: Number(formData.get("bracketWidth")),
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
        // Use the gltf_url from response with cache-busting timestamp
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
    <div className="container mx-auto p-4 lg:p-8 text-center relative z-10">
      <div className="absolute top-4 right-4 lg:top-11 lg:right-11">
        <ThemePicker />
      </div>
      <Card>
        <CardHeader className="gap-4">
          <CardTitle className="text-3xl font-bold">
            Platerator
          </CardTitle>
          <CardDescription>
            Configure your actuator plate specifications to generate STEP and glTF files.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col lg:flex-row gap-4">
          <div className="flex-1 min-w-0">
            <div className="w-full aspect-square min-h-64 lg:min-h-96">
              {modelSrc ? (
                <ModelViewer src={modelSrc} alt="Actuator plate model" />
              ) : (
                <div
                  className="w-full h-full flex items-center justify-center rounded-md border"
                  style={{
                    backgroundColor: "hsl(var(--muted))",
                    borderColor: "hsl(var(--border))",
                  }}
                >
                  <span className="text-muted-foreground">
                    Generate a model to preview it here
                  </span>
                </div>
              )}
            </div>
          </div>
          <div className="flex-1 min-w-0">
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 auto-rows-auto gap-4">
                <Combined
                  forProp="boltSpacing"
                  name="Bolt Spacing"
                  defaultValue="60"
                  validator={validateBoltSpacing}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="boltDiameter"
                  name="Bolt Diameter"
                  defaultValue="10"
                  validator={validateBoltDiameter}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="bracketHeight"
                  name="Bracket Height"
                  defaultValue="400"
                  validator={validateBracketHeight}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="bracketWidth"
                  name="Bracket Width"
                  defaultValue="300"
                  validator={validateBracketWidth}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="pinDiameter"
                  name="Pin Diameter"
                  defaultValue="10"
                  validator={validatePinDiameter}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="pinCount"
                  name="Pin Count"
                  defaultValue="6"
                  validator={validatePinCount}
                  onValidationChange={handleValidationChange}
                />
                <Combined
                  forProp="plateThickness"
                  name="Plate Thickness"
                  defaultValue="8"
                  validator={validatePlateThickness}
                  onValidationChange={handleValidationChange}
                />
              </div>
              <Button type="submit" className="w-full" disabled={isLoading || !isFormValid}>
                {isLoading ? "Generating..." : "Generate Model"}
              </Button>

              {!isFormValid && (
                <p className="text-xs text-muted-foreground text-center">
                  Please fix validation errors before generating the model
                </p>
              )}

              {downloadUrl && (
                <div className="p-4 bg-green-50 dark:bg-green-950 rounded-md border border-green-200 dark:border-green-800">
                  <p className="text-sm text-green-800 dark:text-green-200 mb-2">
                    Model generated successfully!
                  </p>
                  <Button asChild className="w-full">
                    <a href={downloadUrl} download>
                      Download STEP File
                    </a>
                  </Button>
                </div>
              )}

              {errorMessage && (
                <div className="p-4 bg-red-50 dark:bg-red-950 rounded-md border border-red-200 dark:border-red-800">
                  <p className="text-sm text-red-800 dark:text-red-200">
                    {errorMessage}
                  </p>
                </div>
              )}
            </form>
          </div>
        </CardContent>
        <CardFooter>Made in PGH. {new Date().getFullYear()}. AMDG.</CardFooter>
      </Card>
    </div>
  );
}

export default App;
