<script lang="ts">
  import MetricCard from '../ui/MetricCard.svelte';
  import { getStats, isLoading } from '../stores/stats.svelte.ts';
  import Skeleton from '../ui/Skeleton.svelte';

  let stats = $derived(getStats());
  let loading = $derived(isLoading());
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
        <div class="h-full bg-[var(--color-primary)] rounded-full" style="width: 75%"></div>
      </div>
    </MetricCard>
    <MetricCard label="Total Relationships" value={stats.total_edges.toLocaleString()} icon="account_tree">
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-secondary)] rounded-full" style="width: 50%"></div>
      </div>
    </MetricCard>
    <MetricCard
      label="Entity Types"
      value={Object.keys(stats.by_type).length.toString()}
      icon="category"
    >
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-tertiary)] rounded-full" style="width: 60%"></div>
      </div>
    </MetricCard>
    <MetricCard label="Graph Density" icon="hub" value="--">
      <div class="h-1 bg-[var(--color-surface-container-high)] rounded-full overflow-hidden">
        <div class="h-full bg-[var(--color-primary-container)] rounded-full" style="width: 65%"></div>
      </div>
    </MetricCard>
  </div>
{/if}
