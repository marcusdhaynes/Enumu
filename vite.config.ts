import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({ plugins: [svelte()], base: process.env.GITHUB_PAGES === 'true' ? '/Enumu/' : '/', clearScreen: false, server: { strictPort: true, port: 1420 } });
