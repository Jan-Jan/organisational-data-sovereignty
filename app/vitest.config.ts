import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// Not-minted control 1 (app/docs/risk/2026-09-14-app-hazards.md §8): the
// webview's feature tests render components. `sveltekit()` compiles the
// `.svelte` files and resolves `$lib`; `svelteTesting()` resolves Svelte's
// browser build under Vitest and unmounts each rendered component after its
// test. The default environment is `node`, for the pure tests; a test file
// that renders declares `// @vitest-environment jsdom` in its header.
export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	test: {
		environment: 'node',
		include: ['tests/**/*.test.ts']
	}
});
