<script lang="ts">
  import { getCurrentRoute, navigate } from '../router/index.svelte.ts';
  import type { Route } from '../router/index.svelte.ts';

  type NavItem = {
    page: Route['page'];
    icon: string;
    label: string;
  };

  const items: NavItem[] = [
    { page: 'dashboard', icon: 'dashboard', label: 'Insights' },
    { page: 'explorer', icon: 'hub', label: 'Explorer' },
    { page: 'ontology', icon: 'schema', label: 'Ontology' },
  ];

  function isActive(page: string): boolean {
    return getCurrentRoute().page === page;
  }

  function handleNav(page: Route['page']) {
    navigate({ page } as Route);
  }
</script>

<aside class="flex flex-col h-full w-64 border-r shrink-0 overflow-y-auto
  bg-[var(--color-surface-container-lowest)] border-[var(--color-outline-variant)]">
  <div class="p-6">
    <h1 class="text-xl font-bold text-[var(--color-primary)]">Episteme</h1>
    <p class="text-sm text-[var(--color-on-surface-variant)]">Knowledge Graph</p>
  </div>

  <nav class="flex-1 px-4 space-y-1">
    {#each items as item}
      <button
        onclick={() => handleNav(item.page)}
        class="w-full flex items-center gap-3 px-4 py-2.5 rounded-lg transition-colors text-left
          {isActive(item.page)
            ? 'text-[var(--color-primary)] bg-[var(--color-primary-container)]/10 border-r-2 border-[var(--color-primary)]'
            : 'text-[var(--color-on-surface-variant)] hover:bg-[var(--color-surface-container-high)]/50'}"
      >
        <span class="material-symbols-outlined">{item.icon}</span>
        <span class="text-sm">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="p-4 space-y-1 border-t border-[var(--color-outline-variant)]">
    <button disabled title="Coming soon" class="w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm
      text-[var(--color-on-surface-variant)] opacity-50 cursor-not-allowed">
      <span class="material-symbols-outlined">settings</span>
      <span>Settings</span>
    </button>
    <button disabled title="Coming soon" class="w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm
      text-[var(--color-on-surface-variant)] opacity-50 cursor-not-allowed">
      <span class="material-symbols-outlined">help</span>
      <span>Support</span>
    </button>
  </div>
</aside>
