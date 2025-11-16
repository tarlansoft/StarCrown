<script>
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/tauri";
	import { showPopup, query, tags, filteredMessages, selectedTag, messages } from "./store.js";

	let search = "";
	$showPopup; // to subscribe
	$query;
	$tags = [];
	$messages = [];
	$filteredMessages = [];
	$selectedTag = "";

	onMount(async () => {
		await refreshTags();
	});

	async function refreshTags() {
		try {
			const res = await invoke("cmd_list_tags");
			tags.set(res);
		} catch (e) {
			console.error(e);
		}
	}

	$: (() => {
		search = $query;
		if ($query && !$selectedTag && $tags.includes($query)) {
			selectTag($query);
		}
	})();

	async function selectTag(name) {
		selectedTag.set(name);
		try {
			const res = await invoke("cmd_get_messages_for_tag", { tag: name });
			messages.set(res);
			filteredMessages.set(res);
		} catch (e) {
			console.error(e);
		}
	}

	function onSearchInput(e) {
		const val = e.target.value;
		query.set(val);
		if ($selectedTag) {
			const f = $messages.filter(([, text]) =>
				text.toLowerCase().includes(val.toLowerCase())
			);
			filteredMessages.set(f);
		}
	}

	async function chooseMessage(id) {
		try {
			await invoke("cmd_expand_with_message", { messageId: id });
			showPopup.set(false);
		} catch (e) {
			console.error(e);
		}
	}

	function close() {
		showPopup.set(false);
	}
</script>

{#if $showPopup}
	<div class="overlay" on:click|self={close}>
		<div class="popup">
			<div class="header">
				<input
					placeholder="Поиск по тегам или сообщениям..."
					bind:value={search}
					on:input={onSearchInput}
					autofocus
				/>
			</div>
			<div class="body">
				<div class="tags">
					{#each $tags as t}
						<button
							class:selected={t === $selectedTag}
							on:click={() => selectTag(t)}
						>
							/{t}
						</button>
					{/each}
				</div>
				<div class="messages">
					{#if $selectedTag}
						{#each $filteredMessages as [id, text]}
							<div class="message" on:click={() => chooseMessage(id)}>{text}</div>
						{/each}
					{:else}
						<p>Выберите тег слева</p>
					{/if}
				</div>
			</div>
			<div class="footer">
				<span>Enter — вставить • Esc — закрыть</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.25);
		display: grid;
		place-items: center;
		z-index: 9999;
	}
	.popup {
		width: 560px;
		max-width: 92vw;
		background: #111827;
		color: #e5e7eb;
		border-radius: 12px;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.45);
		overflow: hidden;
	}
	.header {
		padding: 12px;
		border-bottom: 1px solid #374151;
	}
	input {
		width: 100%;
		padding: 10px 12px;
		border-radius: 8px;
		border: 1px solid #374151;
		background: #0b1220;
		color: #f3f4f6;
	}
	.body {
		display: grid;
		grid-template-columns: 1fr 2fr;
		gap: 8px;
		padding: 12px;
		max-height: 320px;
	}
	.tags {
		border-right: 1px solid #374151;
		padding-right: 8px;
		overflow: auto;
	}
	.tags button {
		display: block;
		width: 100%;
		text-align: left;
		padding: 8px;
		background: transparent;
		border: none;
		color: #e5e7eb;
		border-radius: 6px;
		cursor: pointer;
	}
	.tags button:hover, .tags button.selected {
		background: #1f2937;
	}
	.messages {
		overflow: auto;
		padding-left: 8px;
	}
	.message {
		padding: 8px;
		border: 1px solid #374151;
		border-radius: 6px;
		margin-bottom: 8px;
		background: #0b1220;
		cursor: pointer;
	}
	.message:hover {
		background: #111827;
	}
	.footer {
		padding: 10px 12px;
		border-top: 1px solid #374151;
		font-size: 12px;
		color: #9ca3af;
	}
</style>


