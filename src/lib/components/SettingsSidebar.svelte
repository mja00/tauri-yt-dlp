<script lang="ts">
    import { onMount } from 'svelte';

    import Button from './Button.svelte';
    import StatusMessage from './StatusMessage.svelte';

    import type { StatusType } from '$lib/types/status';

    import { appStore } from '$lib/stores/appStore';
    import { settingsStore } from '$lib/stores/settingsStore';
    import { openFolderDialog } from '$lib/utils/tauri';



    let saveStatus: string = '';
    let saveStatusType: StatusType = 'muted';
    let saveTimeout: ReturnType<typeof setTimeout> | null = null;

    $: settingsOpen = $appStore.settingsOpen;
    $: downloadLocation = $settingsStore.downloadLocation;

    onMount(() => {
    	settingsStore.load();
    });

    async function handleBrowse(): Promise<void> {
    	try {
    		const selected = await openFolderDialog(downloadLocation);

    		if (selected) {
    			const success = await settingsStore.save(selected);
    			if (success) {
    				saveStatus = 'Download location saved';
    				saveStatusType = 'success';

    				if (saveTimeout) { clearTimeout(saveTimeout); }
    				saveTimeout = setTimeout(() => {
    					if (saveStatus === 'Download location saved') {
    						saveStatus = '';
    						saveStatusType = 'muted';
    					}
    				}, 2000);
    			} else {
    				saveStatus = `Error: ${$settingsStore.error || 'Failed to save location'}`;
    				saveStatusType = 'error';
    			}
    		}
    	} catch (error) {
    		console.error('Failed to open folder dialog:', error);
    		saveStatus = `Error: ${error instanceof Error ? error.message : String(error)}`;
    		saveStatusType = 'error';
    	}
    }

    function handleClose(): void {
    	appStore.closeSettings();
    }

    function handleBackdropClick(e: MouseEvent): void {
    	if (e.target === e.currentTarget) {
    		handleClose();
    	}
    }
</script>

{#if settingsOpen}
    <!-- Backdrop -->
    <div
        class="fixed inset-0 bg-black/50 z-40 transition-opacity duration-300"
        on:click={handleBackdropClick}
        on:keydown={e => e.key === 'Escape' && handleClose()}
        role="button"
        tabindex="0"
        aria-label="Close settings"
    ></div>

    <!-- Sidebar -->
    <aside
        class="fixed right-0 top-0 h-full w-96 bg-dark-card border-l border-dark-border z-50 shadow-2xl transform transition-transform duration-300 ease-in-out"
        class:translate-x-0={settingsOpen}
        class:translate-x-full={!settingsOpen}
    >
        <div class="p-6 h-full flex flex-col">
            <!-- Header -->
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-semibold text-dark-text">Settings</h2>
                <button
                    class="text-dark-text-muted hover:text-dark-text transition-colors"
                    on:click={handleClose}
                    aria-label="Close settings"
                >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto">
                <div class="mb-6">
                    <label for="settingsDownloadLocation" class="block text-sm font-medium text-dark-text mb-2">
                        Download Location:
                    </label>
                    <div class="flex gap-2.5">
                        <input
                            id="settingsDownloadLocation"
                            type="text"
                            value={downloadLocation}
                            on:input={e => settingsStore.setLocation((e.target as HTMLInputElement).value)}
                            on:change={async () => {
                            	if (downloadLocation.trim()) {
                            		const success = await settingsStore.save(downloadLocation.trim());
                            		if (success) {
                            			saveStatus = 'Download location saved';
                            			saveStatusType = 'success';
                            			if (saveTimeout) { clearTimeout(saveTimeout); }
                            			saveTimeout = setTimeout(() => {
                            				if (saveStatus === 'Download location saved') {
                            					saveStatus = '';
                            					saveStatusType = 'muted';
                            				}
                            			}, 2000);
                            		} else {
                            			saveStatus = `Error: ${$settingsStore.error || 'Failed to save location'}`;
                            			saveStatusType = 'error';
                            		}
                            	}
                            }}
                            class="flex-1 px-3.5 py-2.5 text-sm border-2 border-dark-border rounded-lg bg-dark-bg text-dark-text cursor-text placeholder:text-dark-text-placeholder focus:border-primary focus:ring-4 focus:ring-primary/20 outline-none"
                            placeholder="Enter or browse for download folder path..."
                        />
                        <Button onClick={handleBrowse} size="sm">
                            Browse
                        </Button>
                    </div>
                    <StatusMessage status={saveStatus} type={saveStatusType} />
                </div>
            </div>
        </div>
    </aside>
{/if}

<style>
    .translate-x-0 {
        transform: translateX(0);
    }
    .translate-x-full {
        transform: translateX(100%);
    }
</style>
