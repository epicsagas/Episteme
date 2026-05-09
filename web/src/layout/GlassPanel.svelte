<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Component } from 'svelte';

  let {
    header,
    children,
    padding = 'p-6',
  }: {
    header?: Snippet;
    children: Snippet;
    padding?: string;
  } = $props();
</script>

<div class="glass-panel flex flex-col {padding}">
  {#if header}
    <div class="flex justify-between items-center mb-4 pb-3 border-b border-[var(--color-outline-variant)]/30">
      {@render header()}
    </div>
  {/if}
  {@render children()}
</div>
