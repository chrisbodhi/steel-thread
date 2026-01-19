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
import { useState, type FormEvent } from "react";
import { ModelViewer } from "./components/model-viewer";

function Combined({
  forProp,
  name,
  defaultValue = "10",
}: {
  forProp: string;
  name: string;
  defaultValue?: string;
}) {
  return (
    <div>
      <Label htmlFor={forProp}>{name}</Label>
      <Input
        id={forProp}
        type="number"
        name={forProp}
        defaultValue={defaultValue}
        placeholder={defaultValue}
      />
    </div>
  );
}

export function App() {
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [modelSrc, setModelSrc] = useState<string | null>(null);

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
    <div className="container mx-auto p-8 text-center relative z-10">
      <div className="absolute top-11 right-11">
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
        <CardContent className="flex gap-4">
          <div className="flex-1 min-w-0">
            <div className="w-full aspect-square min-h-96">
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
              <div className="grid grid-cols-2 auto-rows-auto gap-4">
                <Combined
                  forProp="boltSpacing"
                  name="Bolt Spacing"
                  defaultValue="60"
                />
                <Combined
                  forProp="boltDiameter"
                  name="Bolt Diameter"
                  defaultValue="10"
                />
                <Combined
                  forProp="bracketHeight"
                  name="Bracket Height"
                  defaultValue="400"
                />
                <Combined
                  forProp="bracketWidth"
                  name="Bracket Width"
                  defaultValue="300"
                />
                <Combined
                  forProp="pinDiameter"
                  name="Pin Diameter"
                  defaultValue="10"
                />
                <Combined
                  forProp="pinCount"
                  name="Pin Count"
                  defaultValue="6"
                />
                <Combined
                  forProp="plateThickness"
                  name="Plate Thickness"
                  defaultValue="8"
                />
              </div>
              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading ? "Generating..." : "Generate Model"}
              </Button>

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
