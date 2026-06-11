<script lang="ts">
  import GraphToolbar from '../explorer/GraphToolbar.svelte';
  import GraphLegend from '../explorer/GraphLegend.svelte';
  import NodeDetailPanel from '../explorer/NodeDetailPanel.svelte';
  import { getSelectedEntity } from '../stores/graph.svelte.ts';

  let selectedEntity = $derived(getSelectedEntity());
</script>

<div class="flex-1 flex h-full overflow-hidden">
  <!-- Graph Canvas (center) -->
  <div class="flex-1 relative bg-[var(--color-surface-container-lowest)] overflow-hidden">
    <GraphToolbar />
    <GraphLegend />
  </div>

  <!-- Detail Panel (right, persistent) -->
  <NodeDetailPanel />
</div>
