import { writable, derived } from "svelte/store";

export const showPopup = writable(false);
export const query = writable("");
export const tags = writable([]);
export const selectedTag = writable("");
export const messages = writable([]);
export const filteredMessages = writable([]);

export function setQuery(q) {
	query.set(q);
}


