<script>
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { base } from '$app/paths';
    import { getTutorials } from '$lib/utils/tutorial';

    const tutorials = getTutorials();

    onMount(() => {
        if (tutorials.length > 0) {
            goto(`${base}/tutorial/${tutorials[0].slug}`, { replaceState: true });
        }
    });
</script>

<div class="flex items-center justify-center h-64">
    <div class="text-neutral-500 animate-pulse">Loading tutorial...</div>
</div>
