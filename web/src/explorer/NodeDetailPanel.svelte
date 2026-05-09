<script lang="ts">
  import { getSelectedEntity, getNeighborsList, clearSelection } from '../stores/graph.svelte.ts';
  import Badge from '../ui/Badge.svelte';
  import { navigate } from '../router/index.svelte.ts';
  import { ENTITY_TYPE_ICONS } from '../api/types.ts';

  let entity = $derived(getSelectedEntity());
  let neighbors = $derived(getNeighborsList());

  function handleNavigate(id: string) {
    navigate({ page: 'entity', id });
  }
</script>

{#if entity}
  <aside class="ml-auto w-96 h-full glass-panel border-l border-[var(--color-outline-variant)] z-40 flex flex-col overflow-hidden">
    <div class="p-6 flex justify-between items-center border-b border-[var(--color-outline-variant)]">
      <h2 class="font-bold text-[var(--color-primary)]">Node Details</h2>
      <button onclick={clearSelection} class="p-1 hover:bg-[var(--color-surface-container-high)] rounded-full transition-colors">
        <span class="material-symbols-outlined text-[var(--color-outline)]">close</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-6 space-y-6">
      <!-- Entity Header -->
      <div class="flex items-start gap-4">
        <div class="w-14 h-14 rounded-lg flex items-center justify-center border"
          style="background: color-mix(in srgb, var(--color-primary) 15%, transparent);
                 border-color: color-mix(in srgb, var(--color-primary) 30%, transparent);">
          <span class="material-symbols-outlined text-[var(--color-primary)] text-2xl">
            {ENTITY_TYPE_ICONS[entity.type]}
          </span>
        </div>
        <div class="flex-1 min-w-0">
          <Badge type={entity.type} small />
          <h3 class="font-bold text-[var(--color-on-surface)] mt-1 leading-tight">{entity.title}</h3>
          <p class="text-xs text-[var(--color-on-surface-variant)] font-mono">{entity.id}</p>
        </div>
      </div>

      <!-- Description -->
      {#if entity.description}
        <p class="text-sm text-[var(--color-on-surface-variant)] leading-relaxed">{entity.description}</p>
      {/if}

      <!-- Category & Tags -->
      {#if entity.category || entity.tags.length > 0}
        <div>
          {#if entity.category}
            <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-1">Category</p>
            <p class="text-sm font-bold text-[var(--color-on-surface)] capitalize">{entity.category}</p>
          {/if}
          {#if entity.tags.length > 0}
            <div class="flex flex-wrap gap-1 mt-2">
              {#each entity.tags.slice(0, 6) as tag}
                <span class="text-[10px] px-2 py-0.5 rounded bg-[var(--color-surface-container-high)] text-[var(--color-on-surface-variant)]">{tag}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Relations -->
      {#if entity.relations}
        <div>
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-2">Relations ({neighbors.length})</p>
          <div class="space-y-1">
            {#each neighbors.slice(0, 10) as neighbor}
              <button
                onclick={() => handleNavigate(neighbor.id)}
                class="w-full flex items-center justify-between p-2 rounded border border-transparent
                  hover:border-[var(--color-outline-variant)] hover:bg-[var(--color-surface-container-high)]/20
                  transition-all cursor-pointer text-left"
              >
                <div class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-[var(--color-tertiary)] text-sm">link</span>
                  <span class="text-sm text-[var(--color-on-surface)]">{neighbor.title}</span>
                </div>
                <span class="material-symbols-outlined text-[var(--color-outline)] text-sm">chevron_right</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer Actions -->
    <div class="p-6 border-t border-[var(--color-outline-variant)] flex gap-4">
      <button
        onclick={() => handleNavigate(entity.id)}
        class="flex-1 border border-[var(--color-outline)] text-[var(--color-on-surface)] py-2 rounded-lg
          font-bold text-sm hover:bg-[var(--color-surface-container-high)] transition-colors"
      >
        View Details
      </button>
    </div>
  </aside>
{/if}
