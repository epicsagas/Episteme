<script lang="ts">
  import { getSelectedEntity, getNeighborsList, clearSelection } from '../stores/graph.svelte.ts';
  import Badge from '../ui/Badge.svelte';
  import { navigate } from '../router/index.svelte.ts';
  import { ENTITY_TYPE_ICONS } from '../api/types.ts';

  let entity = $derived(getSelectedEntity());
  let neighbors = $derived(getNeighborsList());
  let showAllRelations = $state(false);

  const displayedNeighbors = $derived(
    showAllRelations ? neighbors : neighbors.slice(0, 8)
  );

  function handleNavigate(id: string) {
    navigate({ page: 'entity', id });
  }

  function entityTypeColor(type: string): string {
    const map: Record<string, string> = {
      pattern: 'var(--color-pattern)',
      refactoring: 'var(--color-refactoring)',
      law: 'var(--color-law)',
      smell: 'var(--color-smell)',
      insight: 'var(--color-insight)',
    };
    return map[type] || 'var(--color-primary)';
  }
</script>

<aside class="w-[var(--detail-panel-width)] h-full border-l flex flex-col overflow-hidden shrink-0
  bg-[var(--color-surface-container-low)] border-[var(--color-outline-variant)]
  {entity ? 'translate-x-0' : 'translate-x-full'}
  transition-transform duration-[var(--transition-normal)]">

  {#if entity}
    <!-- Header -->
    <div class="px-4 py-3 flex items-center justify-between border-b border-[var(--color-outline-variant)]
      bg-[var(--color-surface-container)]">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-on-surface-variant)]">
        Node Details
      </h2>
      <button onclick={clearSelection}
        class="p-1 rounded hover:bg-[var(--color-surface-container-high)] transition-colors">
        <span class="material-symbols-outlined text-[var(--color-outline)] text-[18px]">close</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <!-- Entity Preview Card -->
      <div class="px-4 py-4 border-b border-[var(--color-outline-variant)]">
        <div class="flex items-start gap-3">
          <div class="w-11 h-11 rounded-lg flex items-center justify-center shrink-0"
            style="background: color-mix(in srgb, var(--color-{entity.type}) 15%, transparent);
                   border: 1px solid color-mix(in srgb, var(--color-{entity.type}) 30%, transparent);">
            <span class="material-symbols-outlined text-xl" style="color: var(--color-{entity.type})">
              {ENTITY_TYPE_ICONS[entity.type]}
            </span>
          </div>
          <div class="flex-1 min-w-0">
            <Badge type={entity.type} small />
            <h3 class="font-semibold text-[var(--color-on-surface)] mt-1 leading-tight text-sm">{entity.title}</h3>
            <p class="text-[11px] text-[var(--color-on-surface-variant)] font-mono mt-0.5">{entity.id}</p>
          </div>
        </div>
      </div>

      <!-- Metrics Row -->
      <div class="grid grid-cols-2 gap-2 px-4 py-3 border-b border-[var(--color-outline-variant)]">
        <div class="rounded-lg p-2.5 bg-[var(--color-surface-container)]">
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)]">Relations</p>
          <p class="text-lg font-bold" style="color: var(--color-{entity.type})">{neighbors.length}</p>
        </div>
        <div class="rounded-lg p-2.5 bg-[var(--color-surface-container)]">
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)]">Type</p>
          <p class="text-sm font-semibold text-[var(--color-on-surface)] capitalize mt-0.5">{entity.type}</p>
        </div>
      </div>

      <!-- Description -->
      {#if entity.description}
        <div class="px-4 py-3 border-b border-[var(--color-outline-variant)]">
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-1">Description</p>
          <p class="text-xs text-[var(--color-on-surface-variant)] leading-relaxed">{entity.description}</p>
        </div>
      {/if}

      <!-- Tags -->
      {#if entity.tags.length > 0}
        <div class="px-4 py-3 border-b border-[var(--color-outline-variant)]">
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-2">Tags</p>
          <div class="flex flex-wrap gap-1">
            {#each entity.tags.slice(0, 8) as tag}
              <span class="text-[10px] px-2 py-0.5 rounded bg-[var(--color-surface-container-high)]
                text-[var(--color-on-surface-variant)] border border-[var(--color-outline-variant)]/50">{tag}</span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Relations -->
      {#if neighbors.length > 0}
        <div class="px-4 py-3">
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-2">
            Connected Entities ({neighbors.length})
          </p>
          <div class="space-y-0.5">
            {#each displayedNeighbors as neighbor}
              <button
                onclick={() => handleNavigate(neighbor.id)}
                class="w-full flex items-center justify-between px-2.5 py-2 rounded
                  hover:bg-[var(--color-surface-container-high)]/50 transition-colors
                  cursor-pointer text-left group"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <div class="w-1.5 h-1.5 rounded-full shrink-0"
                    style="background: var(--color-{neighbor.type})"></div>
                  <span class="text-xs text-[var(--color-on-surface)] truncate">{neighbor.title}</span>
                </div>
                <span class="material-symbols-outlined text-[14px] text-[var(--color-outline)]
                  opacity-0 group-hover:opacity-100 transition-opacity">chevron_right</span>
              </button>
            {/each}
          </div>

          {#if neighbors.length > 8}
            <button
              onclick={() => showAllRelations = !showAllRelations}
              class="w-full mt-2 py-1.5 text-[11px] text-[var(--color-primary)] hover:text-[var(--color-primary-container)]
                transition-colors text-center"
            >
              {showAllRelations ? 'Show less' : `View ${neighbors.length - 8} more`}
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Footer Actions -->
    <div class="px-4 py-3 border-t border-[var(--color-outline-variant)] flex gap-2">
      <button
        onclick={() => handleNavigate(entity.id)}
        class="flex-1 py-2 rounded-lg text-xs font-semibold transition-colors
          bg-[var(--color-primary)] text-[var(--color-on-primary)]
          hover:opacity-90"
      >
        View Details
      </button>
    </div>
  {:else}
    <!-- Empty state -->
    <div class="flex-1 flex flex-col items-center justify-center px-6 text-center">
      <span class="material-symbols-outlined text-[var(--color-outline-variant)] text-4xl mb-3">account_tree</span>
      <p class="text-sm text-[var(--color-on-surface-variant)]">Select a node to view details</p>
      <p class="text-xs text-[var(--color-outline)] mt-1">Click any node in the graph</p>
    </div>
  {/if}
</aside>
