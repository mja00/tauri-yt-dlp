<script lang="ts">
    import type { StatusType } from '$lib/types/status';

    export let status: string = '';
    export let type: StatusType = 'muted';

    let classes: string = '';

    $: {
    	const baseClasses = 'min-h-5 mb-4 text-sm text-center';
    	const typeClasses: Record<StatusType, string> = {
    		muted: 'text-dark-text-muted',
    		primary: 'text-primary',
    		success: 'text-success',
    		error: 'text-error',
    	};

    	classes = `${baseClasses} ${typeClasses[type]}`;
    }
</script>

{#if status}
    <div class={classes}>{status}</div>
{/if}
