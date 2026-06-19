/* ========================================================================== */
/* =============================== SVGO Config ============================== */
/* ========================================================================== */
/*
  CANONICAL TEMPLATE — astro skill.
  Copy this file verbatim to `src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts`
  in any project that needs SVGO. Then import it from `astro.config.mjs`:

      import { svgo_Config } from "./src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts";
      export default defineConfig({
        experimental: { svgo: svgo_Config },
      });

  This file is the source of truth. If a project needs to deviate, update this
  template first (with reasoning) so every other project can follow. The plugin
  overrides below are deliberate — see Areas/icons.md § SVGO config for the WHY
  behind each one.

  Why this lives in a dedicated module:
  - `astro.config.mjs` stays readable and focused on wiring.
  - The SVGO configuration is long, and the WHY behind each setting matters.
  - Reduces risk of accidental edits during unrelated config work.

  Type safety:
  - Astro's `experimental.svgo` option accepts either `true` or an SVGO config.
  - We export the object form here to keep behavior explicit.
*/

import type { Config as svgo_Config_Typ } from "svgo";

/* -------------------------------------------------------------------------- */
/*                            SVGO Configuration                              */
/* -------------------------------------------------------------------------- */
/*
  Design goals:
  - Safe optimizations only.
  - Never break icons (viewBox preservation, no ID mangling).
  - Never merge/flatten paths/groups in ways that can break duotone icons or
    layered SVGs.

  NOTE:
  - The "preset-default" plugin enables many safe transforms, but we override
    specific sub-plugins to avoid breaking icons.
*/
export const svgo_Config: svgo_Config_Typ = {
	// Keep float precision modest to reduce output without visual harm.
	floatPrecision: 2,

	// Run multiple optimization passes (can slightly improve output).
	multipass: true,

	plugins: [
		// Start from SVGO's recommended baseline set.
		"preset-default",

		// NEVER remove viewBox, otherwise icons cannot scale.
		{
			name: "removeViewBox",
			active: false,
		},

		// Prevent renaming/removing IDs.
		// Needed because many icon sets contain clipPaths or masks.
		// Without this override, SVGO may break them.
		{
			name: "cleanupIds",
			active: false,
		},

		// Prevent merging multiple paths into one.
		// This breaks:
		// - Phosphor Duotone icons
		// - Any layered icons
		// - Some animations
		{
			name: "mergePaths",
			active: false,
		},

		// Prevent collapsing groups.
		// Dual-tone icons often rely on <g> layers.
		// Collapsing them destroys the visual structure.
		{
			name: "collapseGroups",
			active: false,
		},

		// --- ADDITIONAL SAFE OPTIMIZATIONS ---
		// These DO NOT harm icons or animations
		"removeXMLNS",

		{
			name: "removeDimensions",
			active: false, // OK because viewBox is preserved
		},
	],
};
