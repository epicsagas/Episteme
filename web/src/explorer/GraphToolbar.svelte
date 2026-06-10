<script lang="ts">
  import GraphCanvas from './GraphCanvas.svelte';

  let canvas: GraphCanvas | undefined = $state();
  let showInsights = $state(false);
  let showIsolated = $state(false);

  function toggleInsights() {
    showInsights = !showInsights;
    canvas?.setShowInsights(showInsights);
  }

  function toggleIsolated() {
    showIsolated = !showIsolated;
    canvas?.setShowIsolated(showIsolated);
  }
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
  <div class="glass-panel p-1 flex flex-col gap-1">
    <button
      onclick={toggleInsights}
      class="p-2 rounded-lg transition-colors flex items-center gap-2 {showInsights ? 'bg-[var(--color-insight)]/20 text-[var(--color-insight)]' : 'text-[var(--color-on-surface-variant)] hover:bg-[var(--color-insight)]/10'}"
      title="{showInsights ? 'Insights 숨기기' : 'Insights 표시'}"
    >
      <span class="material-symbols-outlined text-[20px]">lightbulb</span>
      <span class="text-[10px] uppercase tracking-wider font-semibold">TK</span>
    </button>
    <button
      onclick={toggleIsolated}
      class="p-2 rounded-lg transition-colors flex items-center gap-2 {showIsolated ? 'bg-[var(--color-primary)]/20 text-[var(--color-primary)]' : 'text-[var(--color-on-surface-variant)] hover:bg-[var(--color-primary)]/10'}"
      title="{showIsolated ? '고립 노드 숨기기' : '고립 노드 표시'}"
    >
      <span class="material-symbols-outlined text-[20px]">hub</span>
      <span class="text-[10px] uppercase tracking-wider font-semibold">ALL</span>
    </button>
  </div>
</div>

<div class="absolute inset-0 z-0">
  <GraphCanvas bind:this={canvas} />
</div>
