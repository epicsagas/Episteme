<script lang="ts">
  import DonutChart from '../ui/DonutChart.svelte';
  import { getStats } from '../stores/stats.svelte.ts';
  import { ENTITY_TYPE_COLORS, ENTITY_TYPE_LABELS, ENTITY_TYPE_ICONS } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';

  let stats = $derived(getStats());

  let segments = $derived(
    stats
      ? Object.entries(stats.by_type).map(([type, count]) => ({
          label: ENTITY_TYPE_LABELS[type as EntityType],
          value: count,
          color: ENTITY_TYPE_COLORS[type as EntityType],
        }))
      : [],
  );

  let total = $derived(stats?.total_entities ?? 0);
</script>

<div class="glass-panel p-6 flex flex-col">
  <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-6">
    <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">donut_large</span>
    Entity Distribution
  </h3>

  {#if segments.length > 0}
    <div class="flex-1 flex items-center justify-center">
      <DonutChart
        {segments}
        centerValue={total.toLocaleString()}
        centerLabel="Total"
      />
    </div>

    <div class="mt-6 grid grid-cols-2 gap-2">
      {#each segments as seg}
        <div class="bg-[var(--color-surface-container)] p-3 rounded-lg border border-[var(--color-outline-variant)]/20">
          <p class="text-[11px] text-[var(--color-on-surface-variant)] mb-1">{seg.label}</p>
          <p class="font-bold text-sm" style="color: {seg.color}">{seg.value}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>
