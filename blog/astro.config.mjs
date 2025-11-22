// @ts-check
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
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
