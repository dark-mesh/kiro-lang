<script lang="ts">
	import { page } from '$app/stores';
	import { base } from '$app/paths';
	import { getTutorials } from '$lib/utils/tutorial';

	const tutorials = getTutorials();
</script>

<div class="flex flex-col md:flex-row min-h-screen">
	<!-- Sidebar -->
	<aside class="w-full md:w-60 lg:w-64 border-r overflow-y-auto md:sticky top-14 h-[calc(100vh-3.5rem)]" style="background:#0d0d0d; border-color:#1a1a1a;">
		<div class="p-4">
			<h2 class="text-xs font-bold text-neutral-500 uppercase tracking-widest mb-4 px-2">Chapters</h2>
			<nav class="space-y-0.5">
				{#each tutorials as tutorial}
					<a
						href="{base}/tutorial/{tutorial.slug}"
						class="block px-3 py-2 rounded-md text-sm transition-colors duration-150
						{tutorial.slug && $page.url.pathname.includes(tutorial.slug)
							? 'bg-red-600/10 text-red-400 font-medium'
							: 'text-neutral-500 hover:text-neutral-200 hover:bg-neutral-800/50'}"
					>
						{tutorial.fullTitle}
					</a>
				{/each}
			</nav>
		</div>
	</aside>

	<!-- Content -->
	<main class="flex-1 min-w-0">
		<div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
			<div class="prose prose-invert prose-sm max-w-none" style="--tw-prose-pre-bg:#0d0d0d;">
				<slot />
			</div>
			<!-- Prev / Next -->
			<div class="mt-16 pt-6 border-t flex justify-between" style="border-color:#1a1a1a;">
				{#each [tutorials.findIndex(t => t.slug && $page.url.pathname.includes(t.slug))] as idx}
					{#if idx > 0}
						<a href="{base}/tutorial/{tutorials[idx - 1].slug}" class="group flex items-center gap-2 text-neutral-500 hover:text-white text-sm transition-colors">
							<svg class="w-4 h-4 group-hover:-translate-x-0.5 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>
							<span>{tutorials[idx - 1].title}</span>
						</a>
					{:else}
						<div></div>
					{/if}
					{#if idx >= 0 && idx < tutorials.length - 1}
						<a href="{base}/tutorial/{tutorials[idx + 1].slug}" class="group flex items-center gap-2 text-neutral-500 hover:text-white text-sm transition-colors">
							<span>{tutorials[idx + 1].title}</span>
							<svg class="w-4 h-4 group-hover:translate-x-0.5 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/></svg>
						</a>
					{/if}
				{/each}
			</div>
		</div>
	</main>
</div>
