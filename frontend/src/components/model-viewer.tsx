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

    // React doesn't properly set attributes on custom elements,
    // so we need to set src manually via the ref
    viewer.setAttribute("src", src);

    // Reset loaded state when src changes
    setLoaded(false);

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
      shadow-intensity="1.2"
      environment-image="neutral"
      exposure="1.3"
      className="w-full h-full block"
      style={{
        backgroundColor: "transparent",
        "--poster-color": "transparent",
      } as React.CSSProperties}
    >
      {!loaded && (
        <div
          slot="progress-bar"
          className="absolute inset-0 flex items-center justify-center flex-col gap-6 bg-muted/30 backdrop-blur-sm"
        >
          <div className="relative">
            <div className="w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center">
              <svg
                className="w-8 h-8 text-primary/60 animate-pulse"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                <line x1="12" y1="22.08" x2="12" y2="12" />
              </svg>
            </div>
            <div className="absolute -inset-2 rounded-3xl border border-primary/10 animate-ping" />
          </div>
          <div className="text-center space-y-2">
            <p className="text-sm font-medium text-foreground/80">Loading model</p>
            <div className="w-32 h-1 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-primary/50 via-primary to-primary/50 rounded-full"
                style={{
                  width: "100%",
                  animation: "shimmer 1.5s ease-in-out infinite",
                  backgroundSize: "200% 100%",
                }}
              />
            </div>
          </div>
        </div>
      )}
    </model-viewer>
  );
}
