import { useEffect, useRef, useState } from "react";
import "@google/model-viewer";

// TypeScript declarations for model-viewer custom element
declare global {
  namespace JSX {
    interface IntrinsicElements {
      "model-viewer": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          ref?: React.RefObject<HTMLElement>;
          src?: string;
          alt?: string;
          "auto-rotate"?: boolean;
          "camera-controls"?: boolean;
          "shadow-intensity"?: string;
          "environment-image"?: string;
          exposure?: string;
          ar?: boolean;
          loading?: "auto" | "lazy" | "eager";
          poster?: string;
        },
        HTMLElement
      >;
    }
  }
}

interface ModelViewerProps {
  src: string;
  alt?: string;
}

export function ModelViewer({
  src,
  alt = "a 3D representation of your specified actuator plate",
}: ModelViewerProps) {
  const [loaded, setLoaded] = useState(false);
  const viewerRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer) return;

    // Reset loaded state when src changes
    setLoaded(false);
    console.log("no longer loaded");

    const handleLoad = () => setLoaded(true);
    const handleError = (e: any) => console.error("Model load error:", e);

    viewer.addEventListener("load", handleLoad);
    viewer.addEventListener("error", handleError);

    return () => {
      viewer.removeEventListener("load", handleLoad);
      viewer.removeEventListener("error", handleError);
    };
  }, [src]);

  return (
    <model-viewer
      ref={viewerRef}
      src={src}
      alt={alt}
      auto-rotate
      camera-controls
      shadow-intensity="1"
      environment-image="neutral"
      exposure="1.2"
      className="w-full h-full block rounded-md border"
      style={{
        backgroundColor: "hsl(var(--muted))",
        borderColor: "hsl(var(--border))",
      }}
    >
      {!loaded && (
        <div
          slot="progress-bar"
          className="flex items-center justify-center h-full flex-col gap-4"
        >
          <div className="text-muted-foreground text-lg">
            Loading 3D model...
          </div>
          <div className="w-48 h-2 bg-muted-foreground/20 rounded-full overflow-hidden">
            <div
              className="h-full bg-primary animate-pulse"
              style={{ width: "60%" }}
            />
          </div>
        </div>
      )}
    </model-viewer>
  );
}
