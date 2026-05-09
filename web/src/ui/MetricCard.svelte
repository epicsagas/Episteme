<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    value,
    icon,
    trend,
    trendUp = true,
    children,
  }: {
    label: string;
    value: string;
    icon: string;
    trend?: string;
    trendUp?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div class="glass-panel p-4 flex flex-col">
  <div class="flex justify-between items-start mb-2">
    <span class="text-[11px] uppercase tracking-wider text-[var(--color-on-surface-variant)] font-semibold">{label}</span>
    <span class="material-symbols-outlined text-[var(--color-primary)] text-[20px]">{icon}</span>
  </div>
  <div class="flex items-baseline gap-2">
    <span class="text-2xl font-bold text-[var(--color-on-surface)]">{value}</span>
    {#if trend}
      <span class="text-xs font-bold flex items-center gap-0.5 {trendUp ? 'text-[var(--color-rel-solves)]' : 'text-[var(--color-error)]'}">
        <span class="material-symbols-outlined text-[14px]">{trendUp ? 'trending_up' : 'trending_down'}</span>
        {trend}
      </span>
    {/if}
  </div>
  {#if children}
    <div class="mt-auto pt-2">
      {@render children()}
    </div>
  {/if}
</div>
