<script lang="ts">
    import { onMount } from 'svelte';

    import SettingsSidebar from '$lib/components/SettingsSidebar.svelte';
    import { appStore } from '$lib/stores/appStore';
    import '../styles.css';

    onMount(() => {
    	appStore.loadVersion();
    });
</script>

<div class="bg-background min-h-screen text-foreground overflow-hidden relative">
    <!-- Settings Button - Top corner of entire window -->
    <button
        class="fixed top-4 right-4 z-30 text-muted-foreground hover:text-foreground transition-colors bg-card rounded-lg p-2 shadow-lg border border-border"
        on:click={() => appStore.toggleSettings()}
        aria-label="Open settings"
    >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
    </button>

    <div class="w-full max-w-4xl min-w-[600px] mx-auto px-8 py-8">
        <slot />
    </div>

    <SettingsSidebar />
</div>
