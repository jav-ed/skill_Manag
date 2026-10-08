import { defineConfig } from 'astro/config';
import solid from '@astrojs/solid-js';

// The server answers with a Content-Security-Policy that allows scripts and styles from itself only, so
// nothing may be inlined: no inline <script>, no inline <style>, no data: scripts. `inlineStylesheets` and
// `assetsInlineLimit` keep Vite and Astro from folding small files into the page.
export default defineConfig({
  integrations: [solid()],
  output: 'static',
  build: { format: 'directory', inlineStylesheets: 'never' },
  vite: { build: { assetsInlineLimit: 0 } },
  devToolbar: { enabled: false },
});
