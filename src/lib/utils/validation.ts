// Validate YouTube URL
export function isValidYouTubeUrl(url: string | null | undefined): boolean {
	if (!url || !url.trim()) {
		return false;
	}

	const trimmedUrl = url.trim();

	// Common YouTube URL patterns
	const youtubePatterns = [
		/^https?:\/\/(www\.)?(youtube\.com|youtu\.be)\/.+/, // Standard YouTube URLs
		/^https?:\/\/m\.youtube\.com\/.+/, // Mobile YouTube URLs
		/^https?:\/\/youtube\.com\/shorts\/.+/, // YouTube Shorts
		/^https?:\/\/(www\.)?youtube\.com\/watch\?v=[\w-]+/, // Watch URLs with video ID
		/^https?:\/\/youtu\.be\/[\w-]+/, // Short youtu.be URLs
		/^https?:\/\/(www\.)?youtube\.com\/embed\/[\w-]+/, // Embed URLs
		/^https?:\/\/(www\.)?youtube\.com\/v\/[\w-]+/, // v/ URLs
	];

	// Check if URL matches any pattern
	for (const pattern of youtubePatterns) {
		if (pattern.test(trimmedUrl)) {
			return true;
		}
	}

	return false;
}

