import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [svelte()],
	server: {
		strictPort: true,
		port: 5173
	},
	build: {
		outDir: "dist",
		emptyOutDir: true
	}
});


