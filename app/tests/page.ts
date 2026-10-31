/**
 * The main page, rendered as the user opens the app, with the real child
 * components, over the faked Tauri boundary (./fake-tauri.ts). Only a test
 * file that declares `// @vitest-environment jsdom` uses it.
 */
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';
import { expect } from 'vitest';
import Page from '../src/routes/+page.svelte';
import { tauri } from './fake-tauri';

/** Renders the page and waits until its start has listed the kept admission requests and verified. */
export async function openApp() {
	const page = render(Page);
	await waitFor(() => {
		expect(tauri.commands()).toContain('verify_organisation');
	});
	// The start's promises settle before the test acts.
	await settle();
	return page;
}

/** Presses the tab whose label is `label`. */
export async function openTab(label: string) {
	await fireEvent.click(screen.getByRole('button', { name: label }));
}

/** Gives the page's pending promises a turn to settle. */
export async function settle() {
	await new Promise((resolve) => setTimeout(resolve, 0));
}
