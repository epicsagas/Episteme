<script lang="ts">
  import GraphCanvas from './GraphCanvas.svelte';

  let canvas: GraphCanvas | undefined = $state();
</script>

<div class="absolute left-6 bottom-6 flex flex-col gap-2 z-30">
  <div class="glass-panel p-1 flex flex-col gap-1">
    <button onclick={() => canvas?.zoomIn()} class="p-2 text-[var(--color-on-surface)] hover:bg-[var(--color-primary)]/20 rounded-lg transition-colors">
      <span class="material-symbols-outlined text-[20px]">add</span>
    </button>
    <button onclick={() => canvas?.zoomOut()} class="p-2 text-[var(--color-on-surface)] hover:bg-[var(--color-primary)]/20 rounded-lg transition-colors">
      <span class="material-symbols-outlined text-[20px]">remove</span>
    </button>
    <button onclick={() => canvas?.fit()} class="p-2 text-[var(--color-on-surface)] hover:bg-[var(--color-primary)]/20 rounded-lg transition-colors">
      <span class="material-symbols-outlined text-[20px]">filter_center_focus</span>
    </button>
  </div>
  <div class="glass-panel p-1 flex flex-col gap-1">
    <button onclick={() => canvas?.runLayout('cose')} class="p-2 text-[var(--color-on-surface)] hover:bg-[var(--color-primary)]/20 rounded-lg transition-colors flex items-center gap-2">
      <span class="material-symbols-outlined text-[20px]">account_tree</span>
      <span class="text-[10px] uppercase tracking-wider font-semibold">Cose</span>
    </button>
    <button onclick={() => canvas?.runLayout('circle')} class="p-2 text-[var(--color-on-surface)] hover:bg-[var(--color-primary)]/20 rounded-lg transition-colors flex items-center gap-2">
      <span class="material-symbols-outlined text-[20px]">circle</span>
      <span class="text-[10px] uppercase tracking-wider font-semibold">Circle</span>
    </button>
  </div>
</div>

<div class="absolute inset-0 z-0">
  <GraphCanvas bind:this={canvas} />
</div>
