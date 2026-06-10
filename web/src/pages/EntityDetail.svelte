<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentRoute } from '../router/index.svelte.ts';
  import { selectEntity, getSelectedEntity, getNeighborsList, isLoading } from '../stores/graph.svelte.ts';
  import { navigate } from '../router/index.svelte.ts';
  import Badge from '../ui/Badge.svelte';
  import EmptyState from '../ui/EmptyState.svelte';
  import Skeleton from '../ui/Skeleton.svelte';
  import InsightForm from '../components/InsightForm.svelte';
  import { ENTITY_TYPE_ICONS, ENTITY_TYPE_COLORS } from '../api/types.ts';
  import type { EntityType, Entity } from '../api/types.ts';

  let route = $derived(getCurrentRoute());
  let entity = $derived(getSelectedEntity());
  let neighbors = $derived(getNeighborsList());
  let loading = $derived(isLoading());

  $effect(() => {
    if (route.page === 'entity' && route.id) {
      selectEntity(route.id);
    }
  });

  function handleNavigate(id: string) {
    navigate({ page: 'entity', id });
  }

  let fromPage = $derived(route.page === 'entity' ? (route.from ?? 'explorer') : 'explorer');

  function goBack() {
    navigate({ page: fromPage === 'ontology' ? 'ontology' : 'explorer' });
  }

  function getProvenance(entity: Entity, neighborId: string): string | null {
    const entries = entity.context?.['link_provenance'];
    if (!entries) return null;
    const match = entries.find(e => e.includes(`:${neighborId}:`));
    if (!match) return null;
    if (match.includes('source=auto')) return 'auto';
    if (match.includes('source=manual')) return 'manual';
    if (match.includes('source=suggested')) return 'suggested';
    return null;
  }
</script>

{#if loading && !entity}
  <div class="space-y-6">
    <Skeleton class="h-20" />
    <div class="grid grid-cols-12 gap-6">
      <div class="col-span-4"><Skeleton class="h-64" /></div>
      <div class="col-span-8"><Skeleton class="h-64" /></div>
    </div>
  </div>
{:else if entity}
  <div class="space-y-6">
    <!-- Entity Header -->
    <section class="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-[var(--color-outline-variant)] pb-6">
      <div class="space-y-2">
        <button onclick={goBack} class="flex items-center gap-1 text-xs text-[var(--color-on-surface-variant)] hover:text-[var(--color-primary)] transition-colors mb-2">
          <span class="material-symbols-outlined text-sm">arrow_back</span>
          {fromPage === 'ontology' ? 'Back to Categories' : 'Back to Explorer'}
        </button>
        <div class="flex items-center gap-2" style="color: {ENTITY_TYPE_COLORS[entity.type]}">
          <span class="material-symbols-outlined text-sm">{ENTITY_TYPE_ICONS[entity.type]}</span>
          <span class="text-[11px] uppercase tracking-widest font-semibold">{entity.type}</span>
        </div>
        <h1 class="text-3xl font-bold tracking-tight text-[var(--color-on-surface)]">{entity.title}</h1>
        <p class="text-sm text-[var(--color-on-surface-variant)] max-w-2xl">{entity.description}</p>
      </div>
      <div class="flex items-center gap-4">
        <Badge type={entity.type} label={entity.category} />
        <span class="font-mono text-xs text-[var(--color-on-surface-variant)]">{entity.id}</span>
      </div>
    </section>

    <!-- Content Grid -->
    <div class="grid grid-cols-12 gap-6">
      <!-- Left: Properties -->
      <div class="col-span-12 lg:col-span-4 space-y-6">
        <!-- Properties Card -->
        <div class="glass-panel p-6">
          <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-4">
            <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">list_alt</span>
            Properties
          </h3>
          <div class="space-y-3">
            <div class="flex justify-between items-center py-2 border-b border-[var(--color-outline-variant)]/30">
              <span class="text-sm text-[var(--color-on-surface-variant)]">Type</span>
              <Badge type={entity.type} small />
            </div>
            <div class="flex justify-between items-center py-2 border-b border-[var(--color-outline-variant)]/30">
              <span class="text-sm text-[var(--color-on-surface-variant)]">Category</span>
              <span class="text-sm font-bold text-[var(--color-on-surface)] capitalize">{entity.category}</span>
            </div>
            <div class="flex justify-between items-center py-2 border-b border-[var(--color-outline-variant)]/30">
              <span class="text-sm text-[var(--color-on-surface-variant)]">Relations</span>
              <span class="text-sm font-bold text-[var(--color-secondary)]">{neighbors.length}</span>
            </div>
            {#if entity.tags.length > 0}
              <div class="py-2">
                <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-2">Tags</p>
                <div class="flex flex-wrap gap-1">
                  {#each entity.tags as tag}
                    <span class="text-[10px] px-2 py-0.5 rounded bg-[var(--color-surface-container-high)] text-[var(--color-on-surface-variant)]">{tag}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Context Card -->
        {#if entity.context && (entity.context.benefits?.length || entity.context.when_to_use?.length || entity.context.drawbacks?.length)}
          <div class="glass-panel p-6">
            <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-4">
              <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">info</span>
              Context
            </h3>
            <div class="space-y-4">
              {#if entity.context.benefits?.length}
                <div>
                  <p class="text-[10px] uppercase tracking-wider text-[var(--color-rel-solves)] mb-1">Benefits</p>
                  <ul class="text-xs text-[var(--color-on-surface-variant)] space-y-1">
                    {#each entity.context.benefits.slice(0, 3) as benefit}
                      <li class="flex gap-1.5"><span class="text-[var(--color-rel-solves)]">&#x2022;</span>{benefit}</li>
                    {/each}
                  </ul>
                </div>
              {/if}
              {#if entity.context.when_to_use?.length}
                <div>
                  <p class="text-[10px] uppercase tracking-wider text-[var(--color-primary)] mb-1">When to Use</p>
                  <ul class="text-xs text-[var(--color-on-surface-variant)] space-y-1">
                    {#each entity.context.when_to_use.slice(0, 3) as item}
                      <li class="flex gap-1.5"><span class="text-[var(--color-primary)]">&#x2022;</span>{item}</li>
                    {/each}
                  </ul>
                </div>
              {/if}
              {#if entity.context.drawbacks?.length}
                <div>
                  <p class="text-[10px] uppercase tracking-wider text-[var(--color-error)] mb-1">Drawbacks</p>
                  <ul class="text-xs text-[var(--color-on-surface-variant)] space-y-1">
                    {#each entity.context.drawbacks.slice(0, 3) as drawback}
                      <li class="flex gap-1.5"><span class="text-[var(--color-error)]">&#x2022;</span>{drawback}</li>
                    {/each}
                  </ul>
                </div>
              {/if}
            </div>
          </div>
        {/if}
        <!-- Add Insight Card -->
        <div class="glass-panel p-6">
          <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-4">
            <span class="material-symbols-outlined text-[var(--color-insight)] text-sm">lightbulb</span>
            Add Insight
          </h3>
          <InsightForm oncreated={(id) => handleNavigate(id)} />
        </div>
      </div>

      <!-- Right: Relations -->
      <div class="col-span-12 lg:col-span-8 space-y-6">
        <!-- Relations Grid -->
        <div class="glass-panel overflow-hidden">
          <div class="flex border-b border-[var(--color-outline-variant)] bg-[var(--color-surface-container-low)]/50">
            <button class="px-6 py-3 text-sm font-bold text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]">
              All Relations ({neighbors.length})
            </button>
          </div>
          <div class="p-6">
            {#if neighbors.length === 0}
              <EmptyState icon="link_off" message="No relations found for this entity" />
            {:else}
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                {#each neighbors as neighbor}
                  <button
                    onclick={() => handleNavigate(neighbor.id)}
                    class="flex items-center gap-4 p-4 bg-[var(--color-surface-container-lowest)] rounded-lg
                      border border-[var(--color-outline-variant)]/30 hover:border-[var(--color-primary)]/50
                      transition-all group cursor-pointer text-left"
                  >
                    <div class="w-10 h-10 rounded-full flex items-center justify-center border shrink-0"
                      style="background: color-mix(in srgb, {ENTITY_TYPE_COLORS[neighbor.type]} 10%, transparent);
                             border-color: color-mix(in srgb, {ENTITY_TYPE_COLORS[neighbor.type]} 20%, transparent);">
                      <span class="material-symbols-outlined text-sm"
                        style="color: {ENTITY_TYPE_COLORS[neighbor.type]}">{ENTITY_TYPE_ICONS[neighbor.type]}</span>
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-1.5">
                        <p class="font-bold text-sm text-[var(--color-on-surface)] truncate">{neighbor.title}</p>
                        {#if entity && getProvenance(entity, neighbor.id)}
                          <span class="text-[9px] px-1.5 py-0.5 rounded shrink-0
                            bg-[var(--color-surface-container-high)]
                            text-[var(--color-on-surface-variant)]">
                            {getProvenance(entity, neighbor.id)}
                          </span>
                        {/if}
                      </div>
                      <p class="text-[11px] text-[var(--color-on-surface-variant)]">{neighbor.type}</p>
                    </div>
                    <span class="material-symbols-outlined text-sm text-[var(--color-outline)]
                      group-hover:text-[var(--color-primary)] transition-colors">chevron_right</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
{:else}
  <EmptyState icon="search" message="Select an entity from the Explorer to view details" />
{/if}
