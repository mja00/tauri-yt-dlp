// Tauri API Response Types
export interface YtdlpVersionInfo {
	version: string;
	source: 'path' | 'bundled';
}

export interface VideoInfo {
	title: string;
	duration?: number;
	view_count?: number;
}

export interface VideoFormat {
	format_id: string;
	quality_label: string;
}

// Store State Types
export interface AppState {
	ytdlpVersion: string | null;
	ytdlpSource: 'path' | 'bundled' | null;
	appVersion: string | null;
	loading: boolean;
	settingsOpen: boolean;
}

export interface VideoState {
	url: string;
	title: string;
	formats: VideoFormat[];
	selectedQuality: string;
	loading: boolean;
	error: string | null;
	isValid: boolean;
	showDownloadSection: boolean;
}

export interface DownloadState {
	inProgress: boolean;
	active: boolean;
	outputLines: string[];
	status: string;
	statusType: 'muted' | 'primary' | 'success' | 'error';
	buttonText: string;
	showCancel: boolean;
	showProgress: boolean;
}

export interface SettingsState {
	downloadLocation: string;
	loading: boolean;
	error: string | null;
}

// Store Types
export type Store<T> = {
	subscribe: (callback: (value: T) => void) => () => void;
};

