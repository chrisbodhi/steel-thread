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

import { APITester } from "./APITester";

import "./index.css";
import React, { useState, type FormEvent } from "react";
import "@google/model-viewer";

// Declare model-viewer as a custom element for TypeScript
declare global {
  namespace JSX {
    interface IntrinsicElements {
      "model-viewer": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          src?: string;
          alt?: string;
          "auto-rotate"?: boolean;
          "camera-controls"?: boolean;
          "shadow-intensity"?: string;
          ar?: boolean;
          loading?: "auto" | "lazy" | "eager";
          poster?: string;
        },
        HTMLElement
      >;
    }
  }
}

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

      if (data.success && data.download_url) {
        setDownloadUrl(data.download_url);
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
            Actuator plate picker
          </CardTitle>
          <CardDescription>
            Configure your plate specifications to receive a STEP file.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex gap-4">
          <div style={{ maxWidth: "50%" }}>
            <img
              src="https://i.pinimg.com/originals/35/c0/2b/35c02b534cdbacbea92ae64ee3fe0a1d.png"
              alt="Cat CAD"
            />
          </div>
          <div className="flex-1">
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(2, 1fr)",
                  gridTemplateRows: "repeat(4, auto)",
                  gap: "1rem",
                }}
              >
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
                <div className="flex flex-col gap-4">
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

                  <div className="w-full h-96 bg-gray-100 dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800 flex items-center justify-center">
                    <model-viewer
                      src="/api/download/gltf"
                      alt="Generated actuator plate model"
                      auto-rotate
                      camera-controls
                      shadow-intensity="1"
                      style={{ width: "100%", height: "100%" }}
                    >
                      <div
                        slot="progress-bar"
                        style={{
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          height: "100%",
                          flexDirection: "column",
                          gap: "1rem",
                        }}
                      >
                        <div className="text-gray-600 dark:text-gray-400 text-lg">
                          Loading 3D model...
                        </div>
                        <div className="w-48 h-2 bg-gray-300 dark:bg-gray-700 rounded-full overflow-hidden">
                          <div className="h-full bg-blue-500 animate-pulse" style={{ width: "60%" }}></div>
                        </div>
                      </div>
                    </model-viewer>
                  </div>
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
        <APITester />
        <CardFooter>Made in PGH. {new Date().getFullYear()}. AMDG.</CardFooter>
      </Card>
    </div>
  );
}

export default App;
