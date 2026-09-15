import { defineConfig } from 'vitest/config';

// No sveltekit() plugin and no $lib alias, by choice: these tests import the
// modules under test by relative path, so the runner needs no SvelteKit
// machinery and cannot fail for a reason belonging to the framework rather
// than to the code. When components are rendered here one day (not-minted
// control 1) this file gains the plugin and a DOM environment.
export default defineConfig({
	test: {
		environment: 'node',
		include: ['tests/**/*.test.ts']
	}
});
