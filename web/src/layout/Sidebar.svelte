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
    { page: 'ontology', icon: 'schema', label: 'Categories' },
  ];

  function isActive(page: string): boolean {
    return getCurrentRoute().page === page;
  }

  function handleNav(page: Route['page']) {
    navigate({ page } as Route);
  }
</script>

<aside class="flex flex-col h-full shrink-0 overflow-hidden
  bg-[var(--color-surface-container-lowest)] border-r border-[var(--color-outline-variant)]"
  style="width: var(--sidebar-width);">

  <!-- Brand Header -->
  <div class="px-5 py-4 border-b border-[var(--color-outline-variant)]">
    <div class="flex items-center gap-2.5">
      <div class="w-8 h-8 rounded-lg flex items-center justify-center
        bg-[var(--color-primary)] text-[var(--color-on-primary)]">
        <span class="material-symbols-outlined text-[18px]">hub</span>
      </div>
      <div>
        <h1 class="text-sm font-bold text-[var(--color-on-surface)] leading-tight">Episteme</h1>
        <p class="text-[10px] text-[var(--color-on-surface-variant)]">Knowledge Graph</p>
      </div>
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 px-3 py-3 space-y-0.5">
    {#each items as item}
      <button
        onclick={() => handleNav(item.page)}
        class="w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-left
          {isActive(item.page)
            ? 'text-[var(--color-primary)] bg-[var(--color-primary)]/10 border-l-2 border-[var(--color-primary)]'
            : 'text-[var(--color-on-surface-variant)] hover:bg-[var(--color-surface-container-high)]/50 border-l-2 border-transparent'}"
      >
        <span class="material-symbols-outlined text-[20px]">{item.icon}</span>
        <span class="text-sm font-medium">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Footer -->
  <div class="px-3 py-3 space-y-0.5 border-t border-[var(--color-outline-variant)]">
    <button disabled title="Coming soon"
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm
        text-[var(--color-on-surface-variant)] opacity-40 cursor-not-allowed">
      <span class="material-symbols-outlined text-[20px]">settings</span>
      <span>Settings</span>
    </button>
    <button disabled title="Coming soon"
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm
        text-[var(--color-on-surface-variant)] opacity-40 cursor-not-allowed">
      <span class="material-symbols-outlined text-[20px]">help</span>
      <span>Support</span>
    </button>
  </div>
</aside>
