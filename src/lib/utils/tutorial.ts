
const modules = import.meta.glob('/src/lib/content/tutorial/**/*.md', { eager: true, query: '?raw', import: 'default' }) as Record<string, string>;

export function getTutorials() {
    const tutorials = Object.entries(modules).map(([path, content]) => {
        // Path example: /src/lib/content/tutorial/chapter-00/00_installation.md
        const parts = path.split('/');
        const filename = parts.pop();
        const folder = parts.pop();
        
        // Extract title from first line
        const match = content.match(/^# (.*)/);
        const title = match ? match[1] : (folder || 'Untitled');

        // Create slug from folder name (e.g., chapter-00)
        // We will assume one main markdown file per chapter folder for now
        const slug = folder || 'unknown';

        return {
            slug,
            title: title.replace(/^Chapter \d+: /, ''), // Clean title if needed
            fullTitle: title,
            path,
            order: parseInt((folder || '').split('-')[1] || '999') 
        };
    });

    return tutorials.sort((a, b) => a.order - b.order);
}
