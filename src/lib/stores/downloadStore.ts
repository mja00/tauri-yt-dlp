import { type Writable, writable } from 'svelte/store';

import type { DownloadState } from '$lib/types';

import {
	type ProgressUnlisten,
	cancelDownload,
	downloadVideo,
	setupProgressListener,
} from '$lib/utils/tauri';

const MAX_OUTPUT_LINES = 3;

type DownloadStore = {
	subscribe: Writable<DownloadState>['subscribe'];
	start: (url: string, quality: string) => Promise<void>;
	cancel: () => Promise<void>;
	setStatus: (status: string, type?: 'muted' | 'primary' | 'success' | 'error') => void;
	reset: () => void;
};

function createDownloadStore(): DownloadStore {
	const { subscribe, set, update } = writable<DownloadState>({
		inProgress: false,
		active: false,
		outputLines: [],
		status: '',
		statusType: 'muted',
		buttonText: 'Download Video',
		showCancel: false,
		showProgress: false,
	});

	let progressUnlisten: ProgressUnlisten | null = null;
	let outputLines: string[] = [];

	return {
		subscribe,
		start: async (url: string, quality: string): Promise<void> => {
			// Reset output lines
			outputLines = [];

			set({
				inProgress: true,
				active: true,
				outputLines: [],
				status: 'Downloading video...',
				statusType: 'primary',
				buttonText: 'Downloading...',
				showCancel: true,
				showProgress: false,
			});

			// Setup progress listener
			try {
				progressUnlisten = await setupProgressListener(
					(line: string) => {
						// Only update if download is still active
						update((state) => {
							if (!state.active) { return state; }

							// Create new array with the new line
							const newLines = [...state.outputLines, line];
							if (newLines.length > MAX_OUTPUT_LINES) {
								newLines.shift();
							}

							// Update local array for consistency
							outputLines = [...newLines];

							return {
								...state,
								outputLines: newLines,
								showProgress: true,
							};
						});
					},
					(error: string) => {
						console.error('[DEBUG] Download error event:', error);
					},
				);
			} catch (error) {
				console.error('Failed to setup progress listener:', error);
			}

			try {
				const result = await downloadVideo(url, quality);
				set({
					inProgress: false,
					active: false,
					outputLines: [],
					status: result,
					statusType: 'success',
					buttonText: 'Download Video',
					showCancel: false,
					showProgress: false,
				});

				// Auto-hide progress after 2 seconds
				setTimeout(() => {
					update((state) => {
						return {
							...state,
							showProgress: false,
							outputLines: [],
						};
					});
				}, 2000);
			} catch (error) {
				const errorString = error instanceof Error ? error.message : String(error);
				const isCancelled = errorString.includes('cancelled') || errorString.includes('Cancel');
				set({
					inProgress: false,
					active: false,
					outputLines: [],
					status: isCancelled ? 'Download cancelled' : `Download failed: ${errorString}`,
					statusType: isCancelled ? 'muted' : 'error',
					buttonText: 'Download Video',
					showCancel: false,
					showProgress: false,
				});
			} finally {
				if (progressUnlisten) {
					await progressUnlisten();
					progressUnlisten = null;
				}
				update((state) => {
					return {
						...state,
						inProgress: false,
						active: false,
					};
				});
			}
		},
		cancel: async (): Promise<void> => {
			if (progressUnlisten) {
				await progressUnlisten();
				progressUnlisten = null;
			}

			try {
				await cancelDownload();
				set({
					inProgress: false,
					active: false,
					outputLines: [],
					status: 'Download cancelled',
					statusType: 'muted',
					buttonText: 'Download Video',
					showCancel: false,
					showProgress: false,
				});
			} catch (error) {
				console.error('Failed to cancel download:', error);
			}
		},
		setStatus: (status: string, type: 'muted' | 'primary' | 'success' | 'error' = 'muted'): void => {
			update((state) => { return { ...state, status, statusType: type }; });
		},
		reset: (): void => {
			set({
				inProgress: false,
				active: false,
				outputLines: [],
				status: '',
				statusType: 'muted',
				buttonText: 'Download Video',
				showCancel: false,
				showProgress: false,
			});
			outputLines = [];
		},
	};
}

export const downloadStore = createDownloadStore();

