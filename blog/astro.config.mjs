// @ts-check
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  outDir: '../static/blog',
  base: '/blog',
  markdown: {
    shikiConfig: {
      theme: 'github-dark',
    },
  },
});
