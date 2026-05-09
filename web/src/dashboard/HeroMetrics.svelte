<script lang="ts">
  import MetricCard from '../ui/MetricCard.svelte';
  import { getStats, isLoading } from '../stores/stats.svelte.ts';
  import Skeleton from '../ui/Skeleton.svelte';

  let stats = $derived(getStats());
  let loading = $derived(isLoading());

  function densityPercent(): string {
    if (!stats || stats.total_entities < 2) return '--';
    const maxEdges = stats.total_entities * (stats.total_entities - 1);
    const pct = Math.min(100, Math.round((stats.total_edges / maxEdges) * 100));
    return `${pct}%`;
  }

  function typePercent(): number {
    if (!stats) return 0;
    const filled = Object.values(stats.by_type).filter(v => v > 0).length;
    return Math.round((filled / 5) * 100);
  }

  function edgeRatioPercent(): number {
    if (!stats || stats.total_entities === 0) return 0;
    return Math.min(100, Math.round((stats.total_edges / stats.total_entities) * 10));
  }
</script>

{#if loading && !stats}
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
    {#each Array(4) as _}
      <Skeleton class="h-28" />
    {/each}
  </div>
{:else if stats}
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
    <MetricCard label="Total Entities" value={stats.total_entities.toLocaleString()} icon="data_object">
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-primary)] rounded-full" style="width: {Math.min(100, Math.round(stats.total_entities / 5))}%"></div>
      </div>
    </MetricCard>
    <MetricCard label="Total Relationships" value={stats.total_edges.toLocaleString()} icon="account_tree">
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-secondary)] rounded-full" style="width: {edgeRatioPercent()}%"></div>
      </div>
    </MetricCard>
    <MetricCard
      label="Entity Types"
      value={Object.keys(stats.by_type).length.toString()}
      icon="category"
    >
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-tertiary)] rounded-full" style="width: {typePercent()}%"></div>
      </div>
    </MetricCard>
    <MetricCard label="Graph Density" icon="hub" value={densityPercent()}>
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-primary-container)] rounded-full" style="width: {densityPercent() === '--' ? 0 : parseInt(densityPercent())}%"></div>
      </div>
    </MetricCard>
  </div>
{/if}
