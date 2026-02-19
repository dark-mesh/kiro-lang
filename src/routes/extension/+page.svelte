<script>
	import { base } from '$app/paths';
</script>

<svelte:head>
	<title>Kiro — VS Code Extension</title>
	<meta name="description" content="Official VS Code extension for the Kiro programming language. Syntax highlighting, hover docs, and language configuration." />
</svelte:head>

<div class="max-w-5xl mx-auto px-4 sm:px-6 py-16">
	<!-- Header -->
	<div class="text-center mb-14">
		<h1 class="text-4xl sm:text-5xl font-bold text-white mb-4">VS Code Extension</h1>
		<p class="text-neutral-400 text-lg max-w-2xl mx-auto leading-relaxed">
			Official language support for <span class="text-red-500 font-semibold">Kiro</span> in Visual Studio Code.
			Focused on clear syntax, explicit control flow, and practical host integration.
		</p>
	</div>

	<!-- ─── Feature Cards ─── -->
	<div class="grid grid-cols-1 gap-6 mb-14">
		<!-- Syntax Highlighting -->
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<div class="flex items-center gap-3 mb-5">
				<div class="text-2xl">🎨</div>
				<h2 class="text-xl font-bold text-white">Syntax Highlighting</h2>
			</div>
			<p class="text-neutral-400 mb-5 leading-relaxed">
				Ships with a full <strong class="text-white">TextMate grammar</strong> for <code>.kiro</code> files, covering every aspect of the language:
			</p>
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<div>
					<h4 class="text-sm font-semibold text-neutral-300 mb-2">Declarations</h4>
					<p class="text-neutral-500"><code>fn</code>, <code>var</code>, <code>struct</code>, <code>import</code>, <code>error</code>, <code>rust</code>, <code>pure</code></p>
				</div>
				<div>
					<h4 class="text-sm font-semibold text-neutral-300 mb-2">Control Flow</h4>
					<p class="text-neutral-500"><code>on</code>, <code>off</code>, <code>loop</code>, <code>in</code>, <code>per</code>, <code>break</code>, <code>continue</code>, <code>return</code>, <code>run</code></p>
				</div>
				<div>
					<h4 class="text-sm font-semibold text-neutral-300 mb-2">Type Keywords</h4>
					<p class="text-neutral-500"><code>num</code>, <code>str</code>, <code>bool</code>, <code>void</code>, <code>adr</code>, <code>pipe</code>, <code>list</code>, <code>map</code></p>
				</div>
				<div>
					<h4 class="text-sm font-semibold text-neutral-300 mb-2">Operators & Commands</h4>
					<p class="text-neutral-500"><code>ref</code>, <code>deref</code>, <code>move</code>, <code>give</code>, <code>take</code>, <code>close</code>, <code>at</code>, <code>push</code>, <code>len</code></p>
				</div>
			</div>
		</div>

		<!-- Hover Documentation -->
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<div class="flex items-center gap-3 mb-5">
				<div class="text-2xl">💡</div>
				<h2 class="text-xl font-bold text-white">Hover Documentation</h2>
			</div>
			<p class="text-neutral-400 mb-4 leading-relaxed">
				Built-in hover docs for instant context — no external docs needed:
			</p>
			<ul class="space-y-2 text-neutral-400">
				<li class="flex gap-2"><span class="text-red-500">›</span> Language <strong class="text-neutral-200">keywords</strong> and core constructs</li>
				<li class="flex gap-2"><span class="text-red-500">›</span> <strong class="text-neutral-200">Standard modules</strong> — <code>std_fs</code>, <code>std_env</code>, <code>std_net</code>, <code>std_time</code>, <code>std_io</code></li>
				<li class="flex gap-2"><span class="text-red-500">›</span> <strong class="text-neutral-200">Qualified calls</strong> — hover <code>std_fs.read</code> or <code>std_io.input</code> for inline help</li>
			</ul>
		</div>

		<!-- Language Configuration -->
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<div class="flex items-center gap-3 mb-5">
				<div class="text-2xl">⚙️</div>
				<h2 class="text-xl font-bold text-white">Language Configuration</h2>
			</div>
			<ul class="space-y-2 text-neutral-400">
				<li class="flex gap-2"><span class="text-red-500">›</span> Bracket pairing for <code>&#123;&#125;</code>, <code>()</code>, and <code>[]</code></li>
				<li class="flex gap-2"><span class="text-red-500">›</span> <code>//</code> line comment support</li>
				<li class="flex gap-2"><span class="text-red-500">›</span> Proper language registration for <code>*.kiro</code> files</li>
			</ul>
		</div>
	</div>

	<!-- ─── Installation ─── -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-14">
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<h2 class="text-lg font-bold text-white mb-4">Local Development</h2>
			<ol class="space-y-3 text-neutral-400 list-decimal list-inside">
				<li>Open <code>kiro-vscode/kiro</code> in VS Code.</li>
				<li>Run <code>npm install</code> if needed.</li>
				<li>Press <strong class="text-white">F5</strong> to launch an Extension Development Host.</li>
				<li>Open a <code>.kiro</code> file in the new window.</li>
			</ol>
		</div>
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<h2 class="text-lg font-bold text-white mb-4">VSIX Packaging</h2>
			<ol class="space-y-3 text-neutral-400 list-decimal list-inside">
				<li>Install <code>vsce</code> globally.</li>
				<li>From <code>kiro-vscode/kiro</code>, run <code>vsce package</code>.</li>
				<li>Install the generated <code>.vsix</code> from the VS Code extensions UI.</li>
			</ol>
		</div>
	</div>

	<!-- ─── Usage ─── -->
	<div class="rounded-xl p-8 mb-14" style="background:#111111; border:1px solid #1a1a1a;">
		<h2 class="text-lg font-bold text-white mb-4">Quick Start</h2>
		<p class="text-neutral-400 mb-5 leading-relaxed">Create any file with the <code>.kiro</code> extension and start coding:</p>
		<pre><code>fn main() -> void {"{"} 
    print "Hello, Kiro!"

    var x = 10
    on (x > 5) {"{"}
        print "Greater than 5"
    {"}"} off {"{"}
        print "5 or less"
    {"}"}
{"}"}
</code></pre>
		<p class="text-neutral-500 mt-4 text-sm">
			Hover over keywords (<code>loop</code>, <code>ref</code>, <code>pipe</code>) or standard module members (<code>std_io.input</code>) to view inline docs.
		</p>
	</div>

	<!-- ─── Project Layout ─── -->
	<div class="rounded-xl p-8 mb-14" style="background:#111111; border:1px solid #1a1a1a;">
		<h2 class="text-lg font-bold text-white mb-5">Project Layout</h2>
		<div class="space-y-3">
			{#each [
				{ path: 'package.json', desc: 'Extension manifest and contributions' },
				{ path: 'extension.js', desc: 'Hover provider registration and activation logic' },
				{ path: 'docs/hoverDocs.js', desc: 'Keyword/module hover documentation source' },
				{ path: 'syntaxes/kiro.tmLanguage.json', desc: 'TextMate grammar' },
				{ path: 'language-configuration.json', desc: 'Bracket and comment rules' }
			] as item}
				<div class="flex items-start gap-3">
					<code class="text-red-400 whitespace-nowrap text-sm">{item.path}</code>
					<span class="text-neutral-500">{item.desc}</span>
				</div>
			{/each}
		</div>
	</div>

	<!-- ─── Current Scope & Roadmap ─── -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-14">
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<h2 class="text-lg font-bold text-white mb-4">✅ Current Scope</h2>
			<ul class="space-y-2 text-neutral-400">
				<li class="flex gap-2"><span class="text-green-500">✓</span> Syntax highlighting</li>
				<li class="flex gap-2"><span class="text-green-500">✓</span> Hover help</li>
				<li class="flex gap-2"><span class="text-green-500">✓</span> Baseline language configuration</li>
			</ul>
		</div>
		<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
			<h2 class="text-lg font-bold text-white mb-4">🔮 Roadmap (LSP)</h2>
			<ul class="space-y-2 text-neutral-400">
				<li class="flex gap-2"><span class="text-neutral-600">○</span> Diagnostics from compiler output</li>
				<li class="flex gap-2"><span class="text-neutral-600">○</span> Go-to-definition</li>
				<li class="flex gap-2"><span class="text-neutral-600">○</span> Rename symbol</li>
				<li class="flex gap-2"><span class="text-neutral-600">○</span> Semantic tokens</li>
				<li class="flex gap-2"><span class="text-neutral-600">○</span> Formatting</li>
			</ul>
		</div>
	</div>

	<!-- ─── Contributing ─── -->
	<div class="rounded-xl p-8" style="background:#111111; border:1px solid #1a1a1a;">
		<h2 class="text-lg font-bold text-white mb-4">Contributing</h2>
		<p class="text-neutral-400 mb-4 leading-relaxed">Contributions are welcome. Good first contributions include:</p>
		<ul class="space-y-2 text-neutral-400 mb-5">
			<li class="flex gap-2"><span class="text-red-500">›</span> Improving grammar precision in <code>syntaxes/kiro.tmLanguage.json</code></li>
			<li class="flex gap-2"><span class="text-red-500">›</span> Expanding hover docs in <code>docs/hoverDocs.js</code></li>
			<li class="flex gap-2"><span class="text-red-500">›</span> Tightening language config behavior</li>
			<li class="flex gap-2"><span class="text-red-500">›</span> Keeping docs aligned with language evolution</li>
		</ul>
		<p class="text-neutral-500 text-sm">
			When changing syntax or keywords in the compiler, update this extension in the same PR to avoid tooling drift.
		</p>
	</div>
</div>
