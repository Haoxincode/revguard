import { defineConfig } from "vite";
import { revguardVite } from "@revguard/vite";

// Example only — requires vite installed in a real app.
export default defineConfig({
  plugins: [revguardVite()],
  build: { outDir: "dist" },
});
