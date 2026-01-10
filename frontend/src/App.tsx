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
import { ThemePicker } from "./components/ui/theme-picker";

import { APITester } from "./APITester";

import "./index.css";

function Combined({ forProp, name }: { forProp: string; name: string }) {
  return (
    <div>
      <Label htmlFor={forProp}>{name}</Label>
      <Input
        id={forProp}
        type="number"
        name={forProp}
        defaultValue="10"
        placeholder="10"
      />
    </div>
  );
}

export function App() {
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
          <div>
            <form>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(2, 1fr)",
                  gridTemplateRows: "repeat(3, auto)",
                  gap: "1rem",
                }}
              >
                <Combined forProp="boltSpacing" name="Bolt Spacing" />
                <Combined forProp="boltDiameter" name="Bolt Diameter" />
                <Combined forProp="bracketHeight" name="Bracket Height" />
                <Combined forProp="pinDiameter" name="Pin Diameter" />
                <Combined forProp="plateThickness" name="Plate Thickness" />
              </div>
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
