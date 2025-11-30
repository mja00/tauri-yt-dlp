<script lang="ts">
    import { onMount } from 'svelte';

    import type { StatusType } from '$lib/types/status';

    import { Alert, AlertDescription } from '$lib/components/ui/alert';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
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

    function getAlertVariant(): 'default' | 'destructive' {
    	if (saveStatusType === 'error') { return 'destructive'; }
    	return 'default';
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
        class="fixed right-0 top-0 h-full w-96 bg-card border-l border-border z-50 shadow-2xl transform transition-transform duration-300 ease-in-out"
        class:translate-x-0={settingsOpen}
        class:translate-x-full={!settingsOpen}
    >
        <div class="p-6 h-full flex flex-col">
            <!-- Header -->
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-semibold text-foreground">Settings</h2>
                <button
                    class="text-muted-foreground hover:text-foreground transition-colors"
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
                    <label for="settingsDownloadLocation" class="block text-sm font-medium text-foreground mb-2">
                        Download Location:
                    </label>
                    <div class="flex gap-2.5">
                        <Input
                            id="settingsDownloadLocation"
                            type="text"
                            bind:value={downloadLocation}
                            on:change={async () => {
                            	settingsStore.setLocation(downloadLocation);
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
                            class="flex-1"
                            placeholder="Enter or browse for download folder path..."
                        />
                        <Button onclick={handleBrowse} size="sm">
                            Browse
                        </Button>
                    </div>
                    {#if saveStatus}
                        <Alert variant={getAlertVariant()} class="mt-2">
                            <AlertDescription>
                                {saveStatus}
                            </AlertDescription>
                        </Alert>
                    {/if}
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
