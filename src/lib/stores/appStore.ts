import { type Writable, writable } from 'svelte/store';

import type { AppState } from '$lib/types';

import { getAppVersion, getYtDlpVersion, updateWindowTitle } from '$lib/utils/tauri';

type AppStore = {
	subscribe: Writable<AppState>['subscribe'];
	loadVersion: () => Promise<void>;
	toggleSettings: () => void;
	closeSettings: () => void;
	openSettings: () => void;
};

function createAppStore(): AppStore {
	const { subscribe, set, update } = writable<AppState>({
		ytdlpVersion: null,
		ytdlpSource: null,
		appVersion: null,
		loading: true,
		settingsOpen: false,
	});

	return {
		subscribe,
		loadVersion: async (): Promise<void> => {
			update((state) => { return { ...state, loading: true }; });
			try {
				const versionInfo = await getYtDlpVersion();
				const appVersion = await getAppVersion();

				await updateWindowTitle(versionInfo.version, versionInfo.source, appVersion);

				set({
					ytdlpVersion: versionInfo.version,
					ytdlpSource: versionInfo.source,
					appVersion: appVersion,
					loading: false,
					settingsOpen: false,
				});
			} catch (error) {
				console.error('Failed to get YT-DLP version:', error);
				set({
					ytdlpVersion: null,
					ytdlpSource: null,
					appVersion: null,
					loading: false,
					settingsOpen: false,
				});
			}
		},
		toggleSettings: (): void => {
			update((state) => { return { ...state, settingsOpen: !state.settingsOpen }; });
		},
		closeSettings: (): void => {
			update((state) => { return { ...state, settingsOpen: false }; });
		},
		openSettings: (): void => {
			update((state) => { return { ...state, settingsOpen: true }; });
		},
	};
}

export const appStore = createAppStore();

