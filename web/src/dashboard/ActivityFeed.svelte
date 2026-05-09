<script lang="ts">
  import { getStats } from '../stores/stats.svelte.ts';
  import { ENTITY_TYPE_COLORS, ENTITY_TYPE_ICONS, ENTITY_TYPE_LABELS } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';
  import { getBaseUrl } from '../stores/connection.svelte.ts';
  import { search } from '../api/endpoints.ts';

  type FeedItem = {
    id: string;
    title: string;
    type: EntityType;
    category: string;
  };

  let items: FeedItem[] = $state([]);
  let loading = $state(false);
  let feedError: string | null = $state(null);

  async function loadRecent() {
    loading = true;
    feedError = null;
    try {
      const res = await search(getBaseUrl(), '*', 8);
      items = res.results.map((r) => ({
        id: r.entity_id,
        title: r.title,
        type: r.type,
        category: r.category,
      }));
    } catch (e) {
      feedError = e instanceof Error ? e.message : 'Failed to load recent entities';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (getStats()) loadRecent();
  });
</script>

<div class="glass-panel rounded-xl flex flex-col overflow-hidden">
  <div class="p-4 flex justify-between items-center border-b border-[var(--color-outline-variant)]/30">
    <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 text-sm">
      <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">update</span>
      Recent Entities
    </h3>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#if feedError}
      <div class="p-4 text-xs text-[var(--color-error)]">{feedError}</div>
    {:else}
    <table class="w-full text-left text-sm border-collapse">
      <thead class="sticky top-0 bg-[var(--color-surface-container-high)] z-10">
        <tr class="border-b border-[var(--color-outline-variant)]">
          <th class="p-4 text-[11px] uppercase tracking-wider text-[var(--color-on-surface-variant)] font-semibold">Type</th>
          <th class="p-4 text-[11px] uppercase tracking-wider text-[var(--color-on-surface-variant)] font-semibold">Entity</th>
          <th class="p-4 text-[11px] uppercase tracking-wider text-[var(--color-on-surface-variant)] font-semibold">ID</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-[var(--color-outline-variant)]/20">
        {#each items as item}
          <tr class="hover:bg-[var(--color-surface-container-high)]/20 transition-colors">
            <td class="p-4">
              <span class="inline-flex items-center gap-1 text-xs font-bold"
                style="color: {ENTITY_TYPE_COLORS[item.type]}">
                <span class="material-symbols-outlined text-[14px]">{ENTITY_TYPE_ICONS[item.type]}</span>
                {item.type}
              </span>
            </td>
            <td class="p-4 font-medium text-[var(--color-on-surface)]">{item.title}</td>
            <td class="p-4 font-mono text-xs text-[var(--color-primary)]">{item.id}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {/if}
  </div>
</div>
