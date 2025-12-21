import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "./components/ui/input";
import { ThemePicker } from "./components/ui/theme-picker";

import "./index.css";

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
              <Input type="text" placeholder="Enter your name" />
              <Input type="email" placeholder="Enter your email" />
              <Input type="number" placeholder="Enter your phone number" />
            </form>
          </div>
        </CardContent>
        <CardFooter>Made in PGH. 2025. AMDG.</CardFooter>
      </Card>
    </div>
  );
}

export default App;
