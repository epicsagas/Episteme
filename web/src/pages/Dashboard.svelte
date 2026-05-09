<script lang="ts">
  import { loadStats, getStats, getError } from '../stores/stats.svelte.ts';
  import { getStatus } from '../stores/connection.svelte.ts';
  import HeroMetrics from '../dashboard/HeroMetrics.svelte';
  import GraphHealthDonut from '../dashboard/GraphHealthDonut.svelte';
  import ActivityFeed from '../dashboard/ActivityFeed.svelte';
  import EmptyState from '../ui/EmptyState.svelte';

  let stats = $derived(getStats());
  let status = $derived(getStatus());
  let error = $derived(getError());

  $effect(() => {
    if (status === 'connected' && !stats) loadStats();
  });
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <h2 class="text-lg font-bold text-[var(--color-on-surface)]">Insights Dashboard</h2>
    {#if status !== 'connected'}
      <span class="text-xs text-[var(--color-error)] flex items-center gap-1">
        <span class="material-symbols-outlined text-sm">cloud_off</span>
        API not connected — run <code class="font-mono bg-[var(--color-surface-container-high)] px-1 rounded">epis api</code>
      </span>
    {/if}
  </div>

  {#if error}
    <div class="glass-panel p-4 border-[var(--color-error)]/30">
      <p class="text-sm text-[var(--color-error)]">{error}</p>
    </div>
  {/if}

  <HeroMetrics />

  {#if stats}
    <div class="grid grid-cols-12 gap-6">
      <div class="col-span-12 lg:col-span-4">
        <GraphHealthDonut />
      </div>
      <div class="col-span-12 lg:col-span-8">
        <ActivityFeed />
      </div>
    </div>
  {:else if status === 'connected'}
    <EmptyState icon="hourglass_empty" message="Loading knowledge graph data..." />
  {/if}
</div>
