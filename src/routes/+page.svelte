<script lang="ts">
    import { onMount } from 'svelte';

    import type { AppState, DownloadState, VideoState } from '$lib/types';
    import type { StatusType } from '$lib/types/status';

    import { Alert, AlertDescription } from '$lib/components/ui/alert';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { appStore } from '$lib/stores/appStore';
    import { downloadStore } from '$lib/stores/downloadStore';
    import { settingsStore } from '$lib/stores/settingsStore';
    import { videoStore } from '$lib/stores/videoStore';
    import { openFolderDialog } from '$lib/utils/tauri';


    let urlInput: string = '';
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let saveStatus: string = '';
    let saveStatusType: StatusType = 'muted';
    let saveTimeout: ReturnType<typeof setTimeout> | null = null;

    // Reactive state
    let videoState: VideoState = $videoStore;
    let downloadState: DownloadState = $downloadStore;
    let downloadLocation: string = $settingsStore.downloadLocation;
    let appState: AppState = $appStore;

    $: videoState = $videoStore;
    $: downloadState = $downloadStore;
    $: downloadLocation = $settingsStore.downloadLocation;
    $: appState = $appStore;

    let previousUrlInput: string = '';

    onMount(() => {
    	settingsStore.load();
    });

    // Reactive statement to handle URL input changes (including paste)
    $: if (urlInput !== previousUrlInput) {
    	previousUrlInput = urlInput;
    	if (debounceTimer) {
    		clearTimeout(debounceTimer);
    	}

    	debounceTimer = setTimeout(() => {
    		videoStore.validateAndFetch(urlInput);
    	}, 500);
    }

    function handleUrlInput(): void {
    	if (debounceTimer) {
    		clearTimeout(debounceTimer);
    	}

    	debounceTimer = setTimeout(() => {
    		videoStore.validateAndFetch(urlInput);
    	}, 500);
    }

    function handleUrlBlur(): void {
    	if (debounceTimer) {
    		clearTimeout(debounceTimer);
    	}
    	const trimmed = urlInput.trim();
    	if (trimmed !== videoState.url) {
    		videoStore.validateAndFetch(trimmed);
    	}
    }

    function handleUrlKeypress(e: KeyboardEvent): void {
    	if (e.key === 'Enter') {
    		if (debounceTimer) {
    			clearTimeout(debounceTimer);
    		}
    		const trimmed = urlInput.trim();
    		if (trimmed !== videoState.url) {
    			videoStore.validateAndFetch(trimmed);
    		}
    	}
    }

    async function handleDownload(): Promise<void> {
    	if (!videoState.url || !videoState.showDownloadSection) {
    		downloadStore.setStatus('Please enter a video URL first', 'error');
    		return;
    	}

    	const quality = videoState.selectedQuality === 'best' ? 'best' : videoState.selectedQuality;
    	await downloadStore.start(videoState.url, quality);
    }

    async function handleCancel(): Promise<void> {
    	await downloadStore.cancel();
    }

    async function handleBrowseLocation(): Promise<void> {
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

    function getAlertVariant(): 'default' | 'destructive' {
    	if (videoState.error) { return 'destructive'; }
    	if (downloadState.statusType === 'error') { return 'destructive'; }
    	return 'default';
    }

    function getStatusAlertVariant(): 'default' | 'destructive' {
    	if (saveStatusType === 'error') { return 'destructive'; }
    	return 'default';
    }
</script>

<h1 class="text-foreground mb-5 text-center text-2xl font-semibold">YouTube Video Downloader</h1>

<div class="mb-4">
    <Input
        type="text"
        bind:value={urlInput}
        oninput={handleUrlInput}
        onblur={handleUrlBlur}
        onkeypress={handleUrlKeypress}
        class="w-full"
        placeholder="Enter YouTube video URL..."
        autocomplete="off"
    />
</div>

{#if videoState.error || downloadState.status}
    <Alert variant={getAlertVariant()} class="mb-4">
        <AlertDescription>
            {videoState.error || downloadState.status}
        </AlertDescription>
    </Alert>
{/if}

<div class="bg-card rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-border {videoState.title ? '' : 'text-muted-foreground italic'}">
    {#if videoState.loading}
        <div class="flex items-center gap-2">
            <div class="spinner w-4 h-4 border-2 border-border border-t-primary rounded-full"></div>
            <span class="text-sm text-muted-foreground">Loading...</span>
        </div>
    {:else if videoState.title}
        <h2 class="text-card-foreground text-base font-medium text-center break-words leading-snug">{videoState.title}</h2>
    {:else}
        <h2 class="text-base font-medium text-center break-words leading-snug">Video title will appear here...</h2>
    {/if}
</div>

{#if videoState.showDownloadSection}
    <div class="mt-4 pt-4 border-t border-border">
        <div class="mb-4">
            <label for="qualitySelect" class="block text-sm font-medium text-foreground mb-2">Quality:</label>
            <div class="relative">
                <select
                    id="qualitySelect"
                    value={videoState.selectedQuality}
                    on:change={e => videoStore.setQuality((e.target as HTMLSelectElement).value)}
                    class="quality-select w-full px-3.5 py-2.5 text-sm border-2 border-border rounded-lg bg-background text-foreground cursor-pointer transition-all duration-300 outline-none pr-10 hover:border-primary focus:border-primary focus:ring-4 focus:ring-primary/20 disabled:opacity-60 disabled:cursor-not-allowed disabled:bg-[#252525]"
                    disabled={videoState.loading}
                >
                    <option value="best" class="bg-card text-card-foreground">Best Quality (Default)</option>
                    {#each videoState.formats as format}
                        <option value={format.format_id} class="bg-card text-card-foreground">{format.quality_label}</option>
                    {/each}
                </select>
                {#if videoState.loading}
                    <div class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center justify-center pointer-events-none">
                        <div class="spinner w-4 h-4 border-2 border-border border-t-primary rounded-full"></div>
                    </div>
                {/if}
            </div>
        </div>

        <div class="mb-4">
            <label for="downloadLocation" class="block text-sm font-medium text-foreground mb-2">Download Location:</label>
            <div class="flex gap-2.5">
                <Input
                    id="downloadLocation"
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
                <Button onclick={handleBrowseLocation} size="sm">Browse</Button>
            </div>
            {#if saveStatus}
                <Alert variant={getStatusAlertVariant()} class="mt-2">
                    <AlertDescription>
                        {saveStatus}
                    </AlertDescription>
                </Alert>
            {/if}
        </div>

        {#if downloadState.showProgress}
            <div class="mb-4 w-full">
                <code class="block w-full p-3 bg-background rounded border border-border text-xs text-muted-foreground font-mono whitespace-pre-wrap overflow-x-auto overflow-y-auto max-h-24 break-words">
                    {downloadState.outputLines.join('\n')}
                </code>
            </div>
        {/if}

        <div class="flex gap-2.5">
            <Button
                onclick={handleDownload}
                size="lg"
                class="flex-1 bg-[#27ae60] hover:bg-[#229954] text-white"
                disabled={downloadState.inProgress || !videoState.url}
            >
                {downloadState.buttonText}
            </Button>
            {#if downloadState.showCancel}
                <Button
                    onclick={handleCancel}
                    variant="destructive"
                    size="lg"
                    class="flex-1"
                >
                    Cancel Download
                </Button>
            {/if}
        </div>
    </div>
{/if}

<div class="text-xs text-muted-foreground text-center mt-4 pt-4 border-t border-border flex items-center justify-center gap-2">
    {#if appState.loading}
        <div class="spinner w-3 h-3 border-2 border-border border-t-primary rounded-full"></div>
        <span>Loading version...</span>
    {:else if appState.ytdlpVersion}
        <span>YT-DLP Version: {appState.ytdlpVersion} ({appState.ytdlpSource === 'path' ? 'System PATH' : 'Bundled'})</span>
    {:else}
        <span>YT-DLP Version: Unknown</span>
    {/if}
</div>
