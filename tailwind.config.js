/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#667eea',
          dark: '#5568d3',
        },
        success: '#51cf66',
        error: '#ff6b6b',
        dark: {
          bg: '#1a1a1a',
          card: '#2d2d2d',
          border: '#404040',
          text: '#e0e0e0',
          'text-muted': '#b0b0b0',
          'text-placeholder': '#888',
        },
      },
    },
  },
  plugins: [],
}

