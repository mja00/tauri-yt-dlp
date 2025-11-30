import { type Writable, writable } from 'svelte/store';

import type { VideoFormat, VideoState } from '$lib/types';

import { getVideoFormats, getVideoInfo } from '$lib/utils/tauri';
import { isValidYouTubeUrl } from '$lib/utils/validation';

type VideoStore = {
	subscribe: Writable<VideoState>['subscribe'];
	setUrl: (url: string) => void;
	validateAndFetch: (url: string) => Promise<void>;
	setQuality: (quality: string) => void;
	reset: () => void;
};

function createVideoStore(): VideoStore {
	const { subscribe, set, update } = writable<VideoState>({
		url: '',
		title: '',
		formats: [],
		selectedQuality: 'best',
		loading: false,
		error: null,
		isValid: false,
		showDownloadSection: false,
	});

	let lastFetchedUrl = '';
	const debounceTimer: ReturnType<typeof setTimeout> | null = null;

	return {
		subscribe,
		setUrl: (url: string): void => {
			update((state) => { return { ...state, url }; });
		},
		validateAndFetch: async (url: string): Promise<void> => {
			const trimmedUrl = url.trim();

			// Don't refetch if URL hasn't changed
			if (trimmedUrl === lastFetchedUrl) {
				return;
			}

			// Clear previous debounce
			if (debounceTimer) {
				clearTimeout(debounceTimer);
			}

			// Reset state if empty
			if (!trimmedUrl) {
				set({
					url: '',
					title: '',
					formats: [],
					selectedQuality: 'best',
					loading: false,
					error: null,
					isValid: false,
					showDownloadSection: false,
				});
				lastFetchedUrl = '';
				return;
			}

			// Validate URL
			if (!isValidYouTubeUrl(trimmedUrl)) {
				set({
					url: trimmedUrl,
					title: '',
					formats: [],
					selectedQuality: 'best',
					loading: false,
					error: 'Invalid YouTube URL. Please enter a valid YouTube video URL.',
					isValid: false,
					showDownloadSection: false,
				});
				lastFetchedUrl = '';
				return;
			}

			// Set loading state
			update((state) => {
				return {
					...state,
					url: trimmedUrl,
					loading: true,
					error: null,
					isValid: true,
					showDownloadSection: false,
				};
			});

			lastFetchedUrl = trimmedUrl;

			try {
				const info = await getVideoInfo(trimmedUrl);

				if (info && info.title) {
					// Load formats in parallel or after getting info
					let formats: VideoFormat[] = [];
					try {
						formats = await getVideoFormats(trimmedUrl);
					} catch (formatError) {
						console.error('Failed to load video formats:', formatError);
						// Continue with empty formats array
					}

					set({
						url: trimmedUrl,
						title: info.title,
						formats: formats,
						selectedQuality: 'best',
						loading: false,
						error: null,
						isValid: true,
						showDownloadSection: true,
					});
				} else {
					throw new Error('No title found');
				}
			} catch (error) {
				set({
					url: trimmedUrl,
					title: '',
					formats: [],
					selectedQuality: 'best',
					loading: false,
					error: error instanceof Error ? error.message : String(error),
					isValid: false,
					showDownloadSection: false,
				});
				lastFetchedUrl = ''; // Reset on error so it can retry
			}
		},
		setQuality: (quality: string): void => {
			update((state) => { return { ...state, selectedQuality: quality }; });
		},
		reset: (): void => {
			set({
				url: '',
				title: '',
				formats: [],
				selectedQuality: 'best',
				loading: false,
				error: null,
				isValid: false,
				showDownloadSection: false,
			});
			lastFetchedUrl = '';
		},
	};
}

export const videoStore = createVideoStore();

