import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

const urlInput = document.getElementById('urlInput');
const status = document.getElementById('status');
const videoInfo = document.getElementById('videoInfo');
const ytdlpVersion = document.getElementById('ytdlpVersion');
const versionText = document.getElementById('versionText');
const versionSpinner = document.getElementById('versionSpinner');
const downloadSection = document.getElementById('downloadSection');
const downloadLocation = document.getElementById('downloadLocation');
const browseLocationBtn = document.getElementById('browseLocation');
const downloadBtn = document.getElementById('downloadBtn');
const cancelBtn = document.getElementById('cancelBtn');
const qualitySelect = document.getElementById('qualitySelect');
const qualitySpinner = document.getElementById('qualitySpinner');
const progressContainer = document.getElementById('progressContainer');
const progressOutput = document.getElementById('progressOutput');

let debounceTimer;
let currentVideoUrl = '';
let lastFetchedUrl = '';

// Validate YouTube URL
function isValidYouTubeUrl(url) {
    if (!url || !url.trim()) {
        return false;
    }
    
    const trimmedUrl = url.trim();
    
    // Common YouTube URL patterns
    const youtubePatterns = [
        /^https?:\/\/(www\.)?(youtube\.com|youtu\.be)\/.+/,  // Standard YouTube URLs
        /^https?:\/\/m\.youtube\.com\/.+/,                    // Mobile YouTube URLs
        /^https?:\/\/youtube\.com\/shorts\/.+/,              // YouTube Shorts
        /^https?:\/\/(www\.)?youtube\.com\/watch\?v=[\w-]+/,  // Watch URLs with video ID
        /^https?:\/\/youtu\.be\/[\w-]+/,                     // Short youtu.be URLs
        /^https?:\/\/(www\.)?youtube\.com\/embed\/[\w-]+/,   // Embed URLs
        /^https?:\/\/(www\.)?youtube\.com\/v\/[\w-]+/,       // v/ URLs
    ];
    
    // Check if URL matches any pattern
    for (const pattern of youtubePatterns) {
        if (pattern.test(trimmedUrl)) {
            return true;
        }
    }
    
    return false;
}

// Load YT-DLP version on startup
async function loadYtDlpVersion() {
    // Show spinner initially
    versionSpinner.style.display = 'block';
    versionText.textContent = 'Loading version...';
    
    try {
        const versionInfo = await invoke('get_ytdlp_version');
        
        const sourceLabel = versionInfo.source === 'path' ? 'System PATH' : 'Bundled';
        versionText.textContent = `YT-DLP Version: ${versionInfo.version} (${sourceLabel})`;
        versionSpinner.style.display = 'none';
        
        // Update window title with version
        try {
            const appWindow = getCurrentWindow();
            const sourceLabelShort = versionInfo.source === 'path' ? 'System' : 'Bundled';
			const appVersion = await invoke('get_app_version');
            await appWindow.setTitle(`YT-DLP GUI - ${versionInfo.version} (${sourceLabelShort}) | App: ${appVersion}`);
        } catch (titleError) {
            console.error('Failed to update window title:', titleError);
        }
    } catch (error) {
        console.error('Failed to get YT-DLP version:', error);
        versionText.textContent = 'YT-DLP Version: Unknown';
        versionSpinner.style.display = 'none';
    }
}

// Fetch video info
async function fetchVideoInfo(url) {
    const trimmedUrl = url.trim();
    
    // Don't refetch if URL hasn't changed
    if (trimmedUrl === lastFetchedUrl) {
        return;
    }
    
    if (!trimmedUrl) {
        status.textContent = '';
        status.className = 'min-h-5 mb-4 text-sm text-center text-dark-text-muted';
        videoInfo.innerHTML = '<h2 class="text-base font-medium text-center break-words leading-snug">Video title will appear here...</h2>';
        videoInfo.className = 'bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border text-dark-text-placeholder italic';
        lastFetchedUrl = '';
        currentVideoUrl = '';
        downloadSection.style.display = 'none';
        resetProgress();
        urlInput.classList.remove('border-error', 'ring-error/20');
        urlInput.classList.add('border-dark-border');
        return;
    }

    // Validate YouTube URL
    if (!isValidYouTubeUrl(trimmedUrl)) {
        status.textContent = 'Invalid YouTube URL. Please enter a valid YouTube video URL.';
        status.className = 'min-h-5 mb-4 text-sm text-center text-error';
        videoInfo.innerHTML = '<h2 class="text-base font-medium text-center break-words leading-snug">Video title will appear here...</h2>';
        videoInfo.className = 'bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border text-dark-text-placeholder italic';
        downloadSection.style.display = 'none';
        currentVideoUrl = '';
        lastFetchedUrl = '';
        resetProgress();
        urlInput.classList.remove('border-primary', 'ring-primary/20', 'border-dark-border');
        urlInput.classList.add('border-error', 'ring-4', 'ring-error/20');
        return;
    }

    // Reset border color on valid URL
    urlInput.classList.remove('border-error', 'ring-error/20');
    urlInput.classList.add('border-dark-border');
    lastFetchedUrl = trimmedUrl;
    status.textContent = 'Loading...';
    status.className = 'min-h-5 mb-4 text-sm text-center text-primary';
    videoInfo.innerHTML = '';
    videoInfo.className = 'bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border';

    try {
        const info = await invoke('get_video_info', { url: trimmedUrl });
        
        if (info.title) {
            status.textContent = '';
            status.className = 'min-h-5 mb-4 text-sm text-center text-dark-text-muted';
            videoInfo.innerHTML = `<h2 class="text-dark-text text-base font-medium text-center break-words leading-snug">${info.title}</h2>`;
            videoInfo.className = 'bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border';
            currentVideoUrl = trimmedUrl;
            downloadSection.style.display = 'block';
            
            // Reset progress when loading a new video
            resetProgress();
            
            // Load available formats
            await loadVideoFormats(trimmedUrl);
        } else {
            throw new Error('No title found');
        }
    } catch (error) {
        status.textContent = `Error: ${error}`;
        status.className = 'min-h-5 mb-4 text-sm text-center text-error';
        videoInfo.innerHTML = '<h2 class="text-base font-medium text-center break-words leading-snug">Video title will appear here...</h2>';
        videoInfo.className = 'bg-dark-bg rounded-lg p-4 mb-4 min-h-[50px] flex items-center justify-center border border-dark-border text-dark-text-placeholder italic';
        downloadSection.style.display = 'none';
        currentVideoUrl = '';
        lastFetchedUrl = ''; // Reset on error so it can retry
        resetProgress();
        urlInput.classList.remove('border-primary', 'ring-primary/20', 'border-dark-border');
        urlInput.classList.add('border-error', 'ring-4', 'ring-error/20');
    }
}

// Debounced URL input handler
function handleUrlInput() {
    // Reset border color when user starts typing
    urlInput.classList.remove('border-error', 'ring-error/20');
    urlInput.classList.add('border-dark-border');
    
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        const url = urlInput.value;
        fetchVideoInfo(url);
    }, 500); // 500ms debounce
}

// Event listeners
urlInput.addEventListener('input', handleUrlInput);
urlInput.addEventListener('blur', () => {
    clearTimeout(debounceTimer);
    const url = urlInput.value.trim();
    // Only fetch if URL actually changed
    if (url !== lastFetchedUrl) {
        fetchVideoInfo(url);
    }
});

urlInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        clearTimeout(debounceTimer);
        const url = urlInput.value.trim();
        // Only fetch if URL actually changed
        if (url !== lastFetchedUrl) {
            fetchVideoInfo(url);
        }
    }
});

// Load video formats and populate quality dropdown
async function loadVideoFormats(url) {
    // Show spinner and disable dropdown
    qualitySpinner.style.display = 'flex';
    qualitySelect.disabled = true;
    qualitySelect.innerHTML = '<option value="best">Loading formats...</option>';
    
    try {
        const formats = await invoke('get_video_formats', { url });
        
        // Clear existing options except "Best Quality"
        qualitySelect.innerHTML = '<option value="best">Best Quality (Default)</option>';
        
        // Add format options
        formats.forEach(format => {
            const option = document.createElement('option');
            option.value = format.format_id;
            option.textContent = format.quality_label;
            qualitySelect.appendChild(option);
        });
    } catch (error) {
        console.error('Failed to load video formats:', error);
        // Keep default "best" option if format loading fails
        qualitySelect.innerHTML = '<option value="best">Best Quality (Default)</option>';
    } finally {
        // Hide spinner and enable dropdown
        qualitySpinner.style.display = 'none';
        qualitySelect.disabled = false;
    }
}

// Load download location on startup
async function loadDownloadLocation() {
    try {
        const location = await invoke('get_download_location');
        downloadLocation.value = location;
    } catch (error) {
        console.error('Failed to get download location:', error);
    }
}

// Allow manual path entry and save on change
downloadLocation.addEventListener('change', async () => {
    const path = downloadLocation.value.trim();
    if (path) {
        try {
            await invoke('set_download_location', { path });
            status.textContent = 'Download location saved';
            status.className = 'min-h-5 mb-4 text-sm text-center text-success';
            setTimeout(() => {
                if (status.textContent === 'Download location saved') {
                    status.textContent = '';
                    status.className = 'min-h-5 mb-4 text-sm text-center text-dark-text-muted';
                }
            }, 2000);
        } catch (error) {
            status.textContent = `Error: ${error}`;
            status.className = 'min-h-5 mb-4 text-sm text-center text-error';
        }
    }
});

// Browse button - for now, just make the input editable
browseLocationBtn.addEventListener('click', () => {
    downloadLocation.readOnly = false;
    downloadLocation.focus();
    downloadLocation.select();
});

// Setup progress listener
let progressUnlisten = null;

async function setupProgressListener() {
    console.log('[DEBUG] Setting up progress listener...');
    if (progressUnlisten) {
        console.log('[DEBUG] Cleaning up previous progress listener');
        await progressUnlisten();
    }
    
    console.log('[DEBUG] Registering download-output listener');
    const unlistenProgress = await listen('download-output', (event) => {
        const line = event.payload;
        console.log('[DEBUG] Received output line:', line);
        updateProgress(line);
    });
    
    console.log('[DEBUG] Registering download-error listener');
    const unlistenError = await listen('download-error', (event) => {
        console.error('[DEBUG] Download error event:', event.payload);
    });
    
    progressUnlisten = async () => {
        await unlistenProgress();
        await unlistenError();
    };
    
    console.log('[DEBUG] Progress listener setup complete');
}

let outputLines = []; // Store last 3 lines of output
let downloadActive = false; // Track if download is actively running
const MAX_OUTPUT_LINES = 3;

function updateProgress(line) {
    // Only update progress if download is active
    if (!downloadActive) {
        return;
    }
    
    // Add new line to the array
    outputLines.push(line);
    
    // Keep only the last MAX_OUTPUT_LINES
    if (outputLines.length > MAX_OUTPUT_LINES) {
        outputLines.shift();
    }
    
    // Update the display
    progressOutput.textContent = outputLines.join('\n');
    progressContainer.style.display = 'block';
}

function resetProgress() {
    outputLines = [];
    downloadActive = false;
    progressOutput.textContent = '';
    progressContainer.style.display = 'none';
}

let downloadInProgress = false;

// Cancel download
cancelBtn.addEventListener('click', async () => {
    if (downloadInProgress) {
        try {
            downloadActive = false; // Stop accepting progress updates
            await invoke('cancel_download');
            status.textContent = 'Download cancelled';
            status.className = 'min-h-5 mb-4 text-sm text-center text-dark-text-muted';
            downloadBtn.disabled = false;
            downloadBtn.textContent = 'Download Video';
            cancelBtn.style.display = 'none';
            downloadInProgress = false;
            resetProgress();
        } catch (error) {
            console.error('Failed to cancel download:', error);
        }
    }
});

// Download video
downloadBtn.addEventListener('click', async () => {
    console.log('[DEBUG] Download button clicked');
    
    if (!currentVideoUrl) {
        console.warn('[DEBUG] No video URL set');
        status.textContent = 'Please enter a video URL first';
        status.className = 'min-h-5 mb-4 text-sm text-center text-error';
        return;
    }
    
    console.log('[DEBUG] Starting download for URL:', currentVideoUrl);
    
    // Fully reset progress state before starting new download
    resetProgress();
    downloadActive = true; // Mark download as active
    
    downloadInProgress = true;
    downloadBtn.disabled = true;
    downloadBtn.textContent = 'Downloading...';
    cancelBtn.style.display = 'block';
    status.textContent = 'Downloading video...';
    status.className = 'min-h-5 mb-4 text-sm text-center text-primary';
    
    console.log('[DEBUG] Setting up progress listener...');
    await setupProgressListener();
    console.log('[DEBUG] Progress listener set up');
    
    try {
        // Pass "best" explicitly when best is selected, otherwise pass the format ID
        const selectedQuality = qualitySelect.value === 'best' ? 'best' : qualitySelect.value;
        console.log('[DEBUG] Selected quality:', selectedQuality);
        console.log('[DEBUG] Invoking download_video command...');
        
        const result = await invoke('download_video', { 
            url: currentVideoUrl,
            quality: selectedQuality
        });
        
        console.log('[DEBUG] Download completed:', result);
        status.textContent = result;
        status.className = 'min-h-5 mb-4 text-sm text-center text-success';
        downloadBtn.textContent = 'Download Video';
        cancelBtn.style.display = 'none';
        // Stop accepting progress updates
        downloadActive = false;
        setTimeout(() => {
            resetProgress();
        }, 2000);
    } catch (error) {
        console.error('[DEBUG] Download error:', error);
        if (error.includes('cancelled') || error.includes('Cancel')) {
            status.textContent = 'Download cancelled';
            status.className = 'min-h-5 mb-4 text-sm text-center text-dark-text-muted';
        } else {
            status.textContent = `Download failed: ${error}`;
            status.className = 'min-h-5 mb-4 text-sm text-center text-error';
        }
        downloadBtn.textContent = 'Download Video';
        cancelBtn.style.display = 'none';
        downloadActive = false; // Stop accepting progress updates
        resetProgress();
    } finally {
        console.log('[DEBUG] Download process finished, cleaning up...');
        downloadBtn.disabled = false;
        downloadInProgress = false;
        downloadActive = false; // Ensure download is marked as inactive
        if (progressUnlisten) {
            await progressUnlisten();
            progressUnlisten = null;
        }
        console.log('[DEBUG] Cleanup complete');
    }
});

// Initialize
loadYtDlpVersion();
loadDownloadLocation();

