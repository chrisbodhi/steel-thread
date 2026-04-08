import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "./ui/dialog";

export function AboutButton() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <button
          type="button"
          className="flex items-center gap-1 text-muted-foreground hover:text-primary transition-colors"
          aria-label="What is Platerator?"
        >
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="12" cy="12" r="10" />
            <path d="M12 16v-4" />
            <path d="M12 8h.01" />
          </svg>
          <span className="text-[10px] uppercase tracking-wider">About</span>
        </button>
      </DialogTrigger>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>What is Platerator?</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 text-sm">
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
              small batch production, or standardizing mounting hardware across
              projects.
            </p>
          </div>

          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              What's a linear actuator?
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              A linear actuator is a device that creates motion in a straight
              line, as opposed to rotational motion. Common types include
              electric motor-driven, pneumatic, and hydraulic actuators.
              They're used in everything from industrial automation to
              adjustable furniture.
            </p>
          </div>

          <div>
            <h4 className="font-semibold text-primary mb-1.5">
              Why mounting plates?
            </h4>
            <p className="text-muted-foreground leading-relaxed">
              Actuators need secure attachment points for both the body and the
              extending rod. A well-designed mounting plate provides structural
              support, proper alignment, and standardized bolt patterns. This
              tool automates the tedious CAD work of creating these custom
              brackets.
            </p>
          </div>

          <div className="pt-2 border-t border-border/50">
            <p className="text-[10px] text-muted-foreground/80 uppercase tracking-wider">
              Tip: Start with default values and experiment with the controls.
              The 3D preview updates when you generate a model.
            </p>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
