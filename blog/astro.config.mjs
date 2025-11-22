// @ts-check
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';

// https://astro.build/config
export default defineConfig({
  integrations: [react()],
  site: 'https://andrianov.dev',
  outDir: '../static',
  base: '/',
  trailingSlash: 'always',
  markdown: {
    shikiConfig: {
      theme: 'github-dark',
    },
  },
});
