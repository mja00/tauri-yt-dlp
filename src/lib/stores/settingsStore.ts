import { type Writable, writable } from 'svelte/store';

import type { SettingsState } from '$lib/types';

import { getDownloadLocation, setDownloadLocation } from '$lib/utils/tauri';

type SettingsStore = {
	subscribe: Writable<SettingsState>['subscribe'];
	load: () => Promise<void>;
	save: (path: string) => Promise<boolean>;
	setLocation: (path: string) => void;
};

function createSettingsStore(): SettingsStore {
	const { subscribe, set, update } = writable<SettingsState>({
		downloadLocation: '',
		loading: false,
		error: null,
	});

	return {
		subscribe,
		load: async (): Promise<void> => {
			update((state) => { return { ...state, loading: true, error: null }; });
			try {
				const location = await getDownloadLocation();
				set({ downloadLocation: location, loading: false, error: null });
			} catch (error) {
				update((state) => {
					return {
						...state,
						loading: false,
						error: error instanceof Error ? error.message : String(error),
					};
				});
			}
		},
		save: async (path: string): Promise<boolean> => {
			update((state) => { return { ...state, loading: true, error: null }; });
			try {
				await setDownloadLocation(path);
				update((state) => { return { ...state, downloadLocation: path, loading: false, error: null }; });
				return true;
			} catch (error) {
				update((state) => {
					return {
						...state,
						loading: false,
						error: error instanceof Error ? error.message : String(error),
					};
				});
				return false;
			}
		},
		setLocation: (path: string): void => {
			update((state) => { return { ...state, downloadLocation: path }; });
		},
	};
}

export const settingsStore = createSettingsStore();

