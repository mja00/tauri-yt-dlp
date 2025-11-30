<script lang="ts">
    import { onMount } from 'svelte';

    import type { AppState, DownloadState, VideoState } from '$lib/types';
    import type { StatusType } from '$lib/types/status';

    import Button from '$lib/components/Button.svelte';
    import StatusMessage from '$lib/components/StatusMessage.svelte';
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

    onMount(() => {
    	settingsStore.load();
    });

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

    function getInputClasses(): string {
    	let classes = 'w-full px-4 py-3.5 text-base border-2 rounded-lg bg-dark-bg text-dark-text transition-all duration-300 outline-none placeholder:text-dark-text-placeholder focus:ring-4 focus:ring-primary/20';

    	if (videoState.error && videoState.url) {
    		classes += ' border-error ring-error/20';
    	} else {
    		classes += ' border-dark-border focus:border-primary';
    	}
    	return classes;
    }
</script>

<h1 class="text-dark-text mb-5 text-center text-2xl font-semibold">YouTube Video Downloader</h1>

<div class="mb-4">
    <input
        type="text"
        bind:value={urlInput}
        on:input={handleUrlInput}
        on:blur={handleUrlBlur}
        on:keypress={handleUrlKeypress}
        class={getInputClasses()}
        placeholder="Enter YouTube video URL..."
        autocomplete="off"
    />
</div>

<StatusMessage status={videoState.error || downloadState.status} type={videoState.error ? 'error' : downloadState.statusType} />

<div class="bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border {videoState.title ? '' : 'text-dark-text-placeholder italic'}">
    {#if videoState.loading}
        <div class="flex items-center gap-2">
            <div class="spinner w-4 h-4 border-2 border-dark-border border-t-primary rounded-full"></div>
            <span class="text-sm text-dark-text-muted">Loading...</span>
        </div>
    {:else if videoState.title}
        <h2 class="text-dark-text text-base font-medium text-center break-words leading-snug">{videoState.title}</h2>
    {:else}
        <h2 class="text-base font-medium text-center break-words leading-snug">Video title will appear here...</h2>
    {/if}
</div>

{#if videoState.showDownloadSection}
    <div class="mt-4 pt-4 border-t border-dark-border">
        <div class="mb-4">
            <label for="qualitySelect" class="block text-sm font-medium text-dark-text mb-2">Quality:</label>
            <div class="relative">
                <select
                    id="qualitySelect"
                    value={videoState.selectedQuality}
                    on:change={e => videoStore.setQuality((e.target as HTMLSelectElement).value)}
                    class="quality-select w-full px-3.5 py-2.5 text-sm border-2 border-dark-border rounded-lg bg-dark-bg text-dark-text cursor-pointer transition-all duration-300 outline-none pr-10 hover:border-primary focus:border-primary focus:ring-4 focus:ring-primary/20 disabled:opacity-60 disabled:cursor-not-allowed disabled:bg-[#252525]"
                    disabled={videoState.loading}
                >
                    <option value="best" class="bg-dark-card text-dark-text">Best Quality (Default)</option>
                    {#each videoState.formats as format}
                        <option value={format.format_id} class="bg-dark-card text-dark-text">{format.quality_label}</option>
                    {/each}
                </select>
                {#if videoState.loading}
                    <div class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center justify-center pointer-events-none">
                        <div class="spinner w-4 h-4 border-2 border-dark-border border-t-primary rounded-full"></div>
                    </div>
                {/if}
            </div>
        </div>

        <div class="mb-4">
            <label for="downloadLocation" class="block text-sm font-medium text-dark-text mb-2">Download Location:</label>
            <div class="flex gap-2.5">
                <input
                    id="downloadLocation"
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
                <Button onClick={handleBrowseLocation} size="sm">Browse</Button>
            </div>
            <StatusMessage status={saveStatus} type={saveStatusType} />
        </div>

        {#if downloadState.showProgress}
            <div class="mb-4">
                <code class="block w-full p-3 bg-dark-bg rounded border border-dark-border text-xs text-dark-text-muted font-mono whitespace-pre-wrap overflow-auto max-h-24">
                    {downloadState.outputLines.join('\n')}
                </code>
            </div>
        {/if}

        <div class="flex gap-2.5">
            <Button
                onClick={handleDownload}
                variant="success"
                size="lg"
                fullWidth={true}
                disabled={downloadState.inProgress || !videoState.url}
            >
                {downloadState.buttonText}
            </Button>
            {#if downloadState.showCancel}
                <Button
                    onClick={handleCancel}
                    variant="danger"
                    size="lg"
                    fullWidth={true}
                >
                    Cancel Download
                </Button>
            {/if}
        </div>
    </div>
{/if}

<div class="text-xs text-dark-text-placeholder text-center mt-4 pt-4 border-t border-dark-border flex items-center justify-center gap-2">
    {#if appState.loading}
        <div class="spinner w-3 h-3 border-2 border-dark-border border-t-primary rounded-full"></div>
        <span>Loading version...</span>
    {:else if appState.ytdlpVersion}
        <span>YT-DLP Version: {appState.ytdlpVersion} ({appState.ytdlpSource === 'path' ? 'System PATH' : 'Bundled'})</span>
    {:else}
        <span>YT-DLP Version: Unknown</span>
    {/if}
</div>
