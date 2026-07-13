import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Konfigurimi i Vite për Tauri.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "chrome110",
    minify: "esbuild",
  },
});
