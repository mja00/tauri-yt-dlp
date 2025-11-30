/**
 * Escapes HTML special characters to prevent XSS
 */
function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
}

/**
 * ANSI color code mappings for dark theme
 */
const ANSI_COLORS: Record<number, string> = {
	// Standard colors (30-37)
	30: '#1a1a1a', // Black -> dark background
	31: '#ff6b6b', // Red
	32: '#51cf66', // Green
	33: '#ffd93d', // Yellow
	34: '#667eea', // Blue
	35: '#c44569', // Magenta
	36: '#4ecdc4', // Cyan
	37: '#e0e0e0', // White -> light text

	// Bright colors (90-97)
	90: '#404040', // Bright Black -> muted
	91: '#ff8787', // Bright Red
	92: '#69db7c', // Bright Green
	93: '#ffec8c', // Bright Yellow
	94: '#7c8aff', // Bright Blue
	95: '#d63384', // Bright Magenta
	96: '#66d9ef', // Bright Cyan
	97: '#ffffff', // Bright White
};

const ANSI_BG_COLORS: Record<number, string> = {
	// Standard background colors (40-47)
	40: '#1a1a1a', // Black
	41: '#ff6b6b', // Red
	42: '#51cf66', // Green
	43: '#ffd93d', // Yellow
	44: '#667eea', // Blue
	45: '#c44569', // Magenta
	46: '#4ecdc4', // Cyan
	47: '#e0e0e0', // White

	// Bright background colors (100-107)
	100: '#404040', // Bright Black
	101: '#ff8787', // Bright Red
	102: '#69db7c', // Bright Green
	103: '#ffec8c', // Bright Yellow
	104: '#7c8aff', // Bright Blue
	105: '#d63384', // Bright Magenta
	106: '#66d9ef', // Bright Cyan
	107: '#ffffff', // Bright White
};

interface AnsiState {
	color?: number;
	bgColor?: number;
	bold?: boolean;
	dim?: boolean;
	italic?: boolean;
	underline?: boolean;
}

/**
 * Parses ANSI escape codes and converts them to HTML with inline styles
 * @param text - Text containing ANSI escape codes
 * @returns HTML string with styled spans
 */
export function parseAnsiToHtml(text: string): string {
	// ANSI escape sequence regex: ESC[ followed by codes and 'm'
	// eslint-disable-next-line no-control-regex
	const ansiRegex = /\u001B\[([\d;]*?)m/g;

	let html = '';
	let lastIndex = 0;
	let state: AnsiState = {};
	const stack: AnsiState[] = [];

	function applyState(currentState: AnsiState): string {
		const styles: string[] = [];

		if (currentState.color !== undefined) {
			const color = ANSI_COLORS[currentState.color];
			if (color) {
				styles.push(`color: ${color}`);
			}
		}

		if (currentState.bgColor !== undefined) {
			const bgColor = ANSI_BG_COLORS[currentState.bgColor];
			if (bgColor) {
				styles.push(`background-color: ${bgColor}`);
			}
		}

		if (currentState.bold) {
			styles.push('font-weight: bold');
		}

		if (currentState.dim) {
			styles.push('opacity: 0.7');
		}

		if (currentState.italic) {
			styles.push('font-style: italic');
		}

		if (currentState.underline) {
			styles.push('text-decoration: underline');
		}

		return styles.length > 0 ? ` style="${styles.join('; ')}"` : '';
	}

	function processCodes(codes: string): void {
		if (!codes) {
			// Reset all attributes
			state = {};
			stack.length = 0;
			return;
		}

		const codeArray = codes
			.split(';')
			.map(codeStr => Number.parseInt(codeStr, 10))
			.filter(codeNum => !Number.isNaN(codeNum));

		for (const code of codeArray) {
			if (code === 0) {
				// Reset all
				state = {};
				stack.length = 0;
			} else if (code === 1) {
				state.bold = true;
			} else if (code === 2) {
				state.dim = true;
			} else if (code === 3) {
				state.italic = true;
			} else if (code === 4) {
				state.underline = true;
			} else if (code === 22) {
				// Reset bold/dim
				state.bold = false;
				state.dim = false;
			} else if (code === 23) {
				// Reset italic
				state.italic = false;
			} else if (code === 24) {
				// Reset underline
				state.underline = false;
			} else if (code >= 30 && code <= 37) {
				// Standard foreground colors
				state.color = code;
			} else if (code === 39) {
				// Reset foreground color
				delete state.color;
			} else if (code >= 40 && code <= 47) {
				// Standard background colors
				state.bgColor = code;
			} else if (code === 49) {
				// Reset background color
				delete state.bgColor;
			} else if (code >= 90 && code <= 97) {
				// Bright foreground colors
				state.color = code;
			} else if (code >= 100 && code <= 107) {
				// Bright background colors
				state.bgColor = code;
			}
		}
	}

	let match;
	while ((match = ansiRegex.exec(text)) !== null) {
		// Add text before the ANSI code
		if (match.index > lastIndex) {
			const textBefore = escapeHtml(text.slice(lastIndex, match.index));
			if (textBefore) {
				const style = applyState(state);
				html += style ? `<span${style}>${textBefore}</span>` : textBefore;
			}
		}

		// Process the ANSI codes
		processCodes(match[1]);

		lastIndex = match.index + match[0].length;
	}

	// Add remaining text
	if (lastIndex < text.length) {
		const textAfter = escapeHtml(text.slice(lastIndex));
		if (textAfter) {
			const style = applyState(state);
			html += style ? `<span${style}>${textAfter}</span>` : textAfter;
		}
	}

	return html || escapeHtml(text);
}

/**
 * Adds colors to log lines based on patterns when ANSI codes are not present
 * This is a fallback for when yt-dlp doesn't output ANSI colors
 */
export function addColorToLogLine(line: string): string {
	const escaped = escapeHtml(line);
	
	// Pattern-based coloring for yt-dlp output
	// [download] progress lines
	if (line.includes('[download]')) {
		// Extract percentage if present
		const percentMatch = line.match(/(\d+\.?\d*)%/);
		if (percentMatch) {
			const percent = Number.parseFloat(percentMatch[1]);
			// Color based on progress: red -> yellow -> green
			let color = '#ff6b6b'; // Red for low progress
			if (percent > 33) {
				color = '#ffd93d'; // Yellow for medium progress
			}
			if (percent > 66) {
				color = '#51cf66'; // Green for high progress
			}
			
			// Color the percentage
			return escaped.replace(
				/(\d+\.?\d*)%/,
				`<span style="color: ${color}; font-weight: bold">$1%</span>`
			);
		}
		
		// Color the [download] tag
		return escaped.replace(
			/\[download\]/,
			'<span style="color: #667eea">[download]</span>'
		);
	}
	
	// Error patterns
	if (line.toLowerCase().includes('error') || line.toLowerCase().includes('failed')) {
		return `<span style="color: #ff6b6b">${escaped}</span>`;
	}
	
	// Success patterns
	if (line.toLowerCase().includes('complete') || line.toLowerCase().includes('finished')) {
		return `<span style="color: #51cf66">${escaped}</span>`;
	}
	
	// Warning patterns
	if (line.toLowerCase().includes('warning')) {
		return `<span style="color: #ffd93d">${escaped}</span>`;
	}
	
	// ETA and speed info
	if (line.includes('ETA') || line.includes('MiB/s') || line.includes('KiB/s')) {
		return escaped.replace(
			/(ETA\s+\d+:\d+)/g,
			'<span style="color: #4ecdc4">$1</span>'
		).replace(
			/(\d+\.?\d*)\s*(MiB|KiB)\/s/g,
			'<span style="color: #66d9ef">$1 $2/s</span>'
		);
	}
	
	return escaped;
}

