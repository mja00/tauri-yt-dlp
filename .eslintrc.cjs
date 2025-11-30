module.exports = {
	root: true,
	extends: [
		'@nodecraft/eslint-config/typescript',
	],
	parserOptions: {
		ecmaVersion: 2020,
		sourceType: 'module',
	},
	env: {
		browser: true,
		es2020: true,
		node: true,
	},
	plugins: ['svelte'],
	overrides: [
		{
			files: ['*.svelte'],
			parser: 'svelte-eslint-parser',
			parserOptions: {
				parser: '@typescript-eslint/parser',
			},
			rules: {
				// Svelte-specific rules
				'svelte/no-at-html-tags': 'warn',
				'svelte/no-unused-svelte-ignore': 'error',
				'svelte/valid-compile': 'error',
				// Disable formatting rules for Svelte files (they have their own formatting)
				'@stylistic/indent': 'off',
				'@stylistic/no-mixed-spaces-and-tabs': 'off',
				// Allow function declarations in Svelte script tags
				'no-inner-declarations': 'off',
				// Allow short variable names in event handlers
				'id-length': ['error', { exceptions: ['e', 'i', 'j', 'k', 'x', 'y'] }],
				// Allow multiple statements per line in template bindings
				'@stylistic/max-statements-per-line': 'off',
			},
		},
	],
	ignorePatterns: [
		'build/',
		'.svelte-kit/',
		'dist/',
		'node_modules/',
		'*.config.js',
		'*.config.cjs',
		'src/lib/components/ui/**',
	],
};

