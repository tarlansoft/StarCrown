<script>
	import Popup from "./Popup.svelte";
	import { onMount } from "svelte";
	import { listen } from "@tauri-apps/api/event";
	import { register, isRegistered } from "@tauri-apps/api/globalShortcut";
	import { showPopup, setQuery } from "./store.js";

	onMount(async () => {
		await listen("show_popup", (e) => {
			const q = e?.payload?.query ?? "";
			setQuery(q);
			showPopup.set(true);
		});

		// Регистрация глобального хоткея Ctrl+Space (через встроенный API Tauri)
		try {
			const hotkey = "Ctrl+Space";
			const reg = await isRegistered(hotkey);
			if (!reg) {
				await register(hotkey, () => {
					setQuery("");
					showPopup.set(true);
				});
			}
		} catch (e) {
			console.error("Global shortcut error", e);
		}
	});
</script>

<main>
	<h1>GoldenCrown Expander</h1>
	<p>Глобальные хоткеи и автозамена активны в фоне.</p>
	<Popup />
</main>

<style>
	main {
		font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial,
			"Apple Color Emoji", "Segoe UI Emoji";
		padding: 16px;
	}
</style>


