import { mdsvex } from 'mdsvex';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		paths: {
            base: process.env.NODE_ENV === 'production' ? '/kiro-lang' : '',
        },
		prerender: {
			handleHttpError: ({ path, referrer, message }) => {
				// Ignore links to .md files from rendered markdown content
				if (path.endsWith('.md')) {
					return;
				}
				throw new Error(message);
			}
		},
		// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://svelte.dev/docs/kit/adapters for more information about adapters.
		adapter: adapter({
			fallback: '404.html'
		})
	},
	appDir: 'internal',
	preprocess: [vitePreprocess(),mdsvex()],
	extensions: ['.svelte', '.svx']
};

export default config;
