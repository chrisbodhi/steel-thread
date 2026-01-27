import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function AboutSection() {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <Card className="backdrop-blur-xl bg-card/80 border-border/50 shadow-lg">
      <CardHeader className="pb-3">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="flex items-center justify-between w-full text-left group"
        >
          <div className="flex items-center gap-2">
            <svg
              className="w-5 h-5 text-primary"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4" />
              <path d="M12 8h.01" />
            </svg>
            <CardTitle className="text-sm lg:text-base font-semibold">
              What is Platerator?
            </CardTitle>
          </div>
          <svg
            className={`w-4 h-4 text-muted-foreground transition-transform group-hover:text-primary ${
              isExpanded ? "rotate-180" : ""
            }`}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      </CardHeader>

      {isExpanded && (
        <CardContent className="space-y-4 text-xs lg:text-sm animate-in fade-in slide-in-from-top-2 duration-200">
          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              Automated Mounting Plate Design
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              Platerator generates custom mounting plates for linear actuators.
              Configure dimensions, materials, and fasteners, then download
              production-ready CAD files (STEP format) and preview models (glTF).
            </p>
          </div>

          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              Who is this for?
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              Mechanical engineers, robotics developers, and manufacturers who
              need custom actuator mounting solutions. Perfect for prototyping,
              small batch production, or standardizing mounting hardware across projects.
            </p>
          </div>

          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              What's a linear actuator?
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              A linear actuator is a device that creates motion in a straight line,
              as opposed to rotational motion. Common types include electric motor-driven,
              pneumatic, and hydraulic actuators. They're used in everything from
              industrial automation to adjustable furniture.
            </p>
          </div>

          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              Why mounting plates?
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              Actuators need secure attachment points for both the body and the
              extending rod. A well-designed mounting plate provides structural
              support, proper alignment, and standardized bolt patterns. This tool
              automates the tedious CAD work of creating these custom brackets.
            </p>
          </div>

          <div className="pt-2 border-t border-border/50">
            <p className="text-[10px] text-muted-foreground/80 uppercase tracking-wider">
              💡 Tip: Start with default values and experiment with the controls.
              The 3D preview updates when you generate a model.
            </p>
          </div>
        </CardContent>
      )}
    </Card>
  );
}
