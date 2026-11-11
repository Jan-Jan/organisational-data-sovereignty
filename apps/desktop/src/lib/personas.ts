/**
 * SDD-23vjd8: what the Persona list shows as a Persona's Handle.
 */

/** LLR-q735zx: the label shown for a Persona that holds no Handle yet, such as a provisional Member. */
export const NO_HANDLE_YET = '(no Handle yet)';

/** LLR-q735zx: the Handle `list_personas` reported, or the stated label when it reported none. */
export function handleLabel(handle: string | null): string {
	return handle ?? NO_HANDLE_YET;
}
