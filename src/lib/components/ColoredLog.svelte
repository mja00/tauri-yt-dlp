<script lang="ts">
	import { parseAnsiToHtml, addColorToLogLine } from '$lib/utils/ansi';

	interface Props {
		lines: string[];
	}

	const { lines = [] }: Props = $props();

	function renderLine(line: string): string {
		// First try parsing ANSI codes
		let html = parseAnsiToHtml(line);
		
		// If no ANSI codes were found (no color spans), add colors based on patterns
		if (!html.includes('<span')) {
			html = addColorToLogLine(line);
		}
		
		return html;
	}
</script>

<div class="colored-log">
	{#each lines as line, index}
		{@html renderLine(line)}
		{#if index < lines.length - 1}
			<br />
		{/if}
	{/each}
</div>

<style>
	.colored-log {
		font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono',
			monospace;
		white-space: pre-wrap;
		word-break: break-word;
		line-height: 1.5;
	}
</style>

