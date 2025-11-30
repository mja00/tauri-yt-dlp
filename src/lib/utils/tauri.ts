import { invoke } from '@tauri-apps/api/core';
import { type Event, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';

import type { VideoFormat, VideoInfo, YtdlpVersionInfo } from '$lib/types';

// Tauri API wrappers
export async function getYtDlpVersion(): Promise<YtdlpVersionInfo> {
	return await invoke<YtdlpVersionInfo>('get_ytdlp_version');
}

export async function getAppVersion(): Promise<string> {
	return await invoke<string>('get_app_version');
}

export async function getVideoInfo(url: string): Promise<VideoInfo> {
	return await invoke<VideoInfo>('get_video_info', { url });
}

export async function getVideoFormats(url: string): Promise<VideoFormat[]> {
	return await invoke<VideoFormat[]>('get_video_formats', { url });
}

export async function downloadVideo(url: string, quality: string): Promise<string> {
	return await invoke<string>('download_video', { url, quality });
}

export async function cancelDownload(): Promise<void> {
	return await invoke<void>('cancel_download');
}

export async function getDownloadLocation(): Promise<string> {
	return await invoke<string>('get_download_location');
}

export async function setDownloadLocation(path: string): Promise<void> {
	return await invoke<void>('set_download_location', { path });
}

export async function openFolderDialog(defaultPath?: string): Promise<string | null> {
	const selected = await open({
		directory: true,
		multiple: false,
		defaultPath: defaultPath || undefined,
	});

	if (typeof selected === 'string') {
		return selected;
	}

	return null;
}

export type ProgressUnlisten = () => Promise<void>;

export async function setupProgressListener(
	onProgress: (line: string) => void,
	onError: (error: string) => void,
): Promise<ProgressUnlisten> {
	const unlistenProgress = await listen<string>('download-output', (event: Event<string>) => {
		onProgress(event.payload);
	});

	const unlistenError = await listen<string>('download-error', (event: Event<string>) => {
		onError(event.payload);
	});

	return async () => {
		await unlistenProgress();
		await unlistenError();
	};
}

export async function updateWindowTitle(
	ytdlpVersion: string,
	source: 'path' | 'bundled',
	appVersion: string,
): Promise<void> {
	try {
		const appWindow = getCurrentWindow();
		const sourceLabelShort = source === 'path' ? 'System' : 'Bundled';
		await appWindow.setTitle(`YT-DLP GUI - ${ytdlpVersion} (${sourceLabelShort}) | App: ${appVersion}`);
	} catch (error) {
		console.error('Failed to update window title:', error);
	}
}

