<script lang="ts">
	import { page } from '$app/stores';
	import { marked } from 'marked';

	// Import all tutorial markdown files as raw strings
	const mdModules = import.meta.glob('/src/lib/content/tutorial/**/*.md', {
		eager: true,
		query: '?raw',
		import: 'default'
	}) as Record<string, string>;

	// Import all .kiro script files as raw strings
	const kiroModules = import.meta.glob('/src/lib/content/tutorial/**/*.kiro', {
		eager: true,
		query: '?raw',
		import: 'default'
	}) as Record<string, string>;

	// Import all .rs files as raw strings (for host chapters)
	const rsModules = import.meta.glob('/src/lib/content/tutorial/**/*.rs', {
		eager: true,
		query: '?raw',
		import: 'default'
	}) as Record<string, string>;

	$: slug = $page.params.slug;
	$: rawContent = Object.entries(mdModules).find(([path]) => path.includes(`/${slug}/`))?.[1] || '';
	$: html = marked(rawContent);

	// Get all script files for this chapter
	$: chapterFiles = [
		...Object.entries(kiroModules)
			.filter(([path]) => path.includes(`/${slug}/`))
			.map(([path, content]) => ({
				name: path.split('/').pop() || '',
				content,
				lang: 'kiro'
			})),
		...Object.entries(rsModules)
			.filter(([path]) => path.includes(`/${slug}/`))
			.map(([path, content]) => ({
				name: path.split('/').pop() || '',
				content,
				lang: 'rust'
			}))
	];
</script>

<svelte:head>
	<title>Kiro Tutorial - {slug}</title>
</svelte:head>

{#if rawContent}
	{@html html}

	<!-- Chapter Script Files -->
	{#if chapterFiles.length > 0}
		<div class="mt-12 border-t pt-8" style="border-color:#1a1a1a;">
			<h3 class="mb-2 flex items-center gap-2 text-xl font-bold text-white">
				<span class="text-red-500">📄</span> Chapter Scripts
			</h3>
			<p class="mb-6 text-sm text-neutral-500">
				Full source code for this chapter. Run with <code>kiro filename.kiro</code>
			</p>

			<div class="space-y-4">
				{#each chapterFiles as file}
					<details
						class="group overflow-hidden rounded-xl"
						style="background:#0d0d0d; border:1px solid #1a1a1a;"
					>
						<summary
							class="flex cursor-pointer items-center justify-between px-5 py-3.5 transition-colors select-none hover:bg-neutral-800/50"
						>
							<div class="flex items-center gap-3">
								<span class="font-mono text-sm font-medium text-red-400">{file.name}</span>
								{#if file.lang === 'rust'}
									<span
										class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
										style="background:#1a1a1a; color:#f97316;">Rust</span
									>
								{/if}
							</div>
							<svg
								class="h-4 w-4 text-neutral-500 transition-transform duration-200 group-open:rotate-180"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M19 9l-7 7-7-7"
								/>
							</svg>
						</summary>
						<div class="border-t" style="border-color:#1a1a1a;">
							<pre class="!m-0 !rounded-none !border-0"><code>{file.content}</code></pre>
						</div>
					</details>
				{/each}
			</div>
		</div>
	{/if}
{:else}
	<div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4 text-red-400">
		Chapter not found. <code>{slug}</code>
	</div>
{/if}
