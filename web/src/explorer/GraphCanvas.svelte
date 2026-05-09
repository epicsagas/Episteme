<script lang="ts">
  import cytoscape from 'cytoscape';
  import type { Core, NodeSingular } from 'cytoscape';
  import { onMount, onDestroy } from 'svelte';
  import { loadFullGraph, getGraphData, isLoading, getError, getVersion } from '../stores/graph.svelte.ts';
  import { selectEntity } from '../stores/graph.svelte.ts';
  import { waitForReady } from '../stores/connection.svelte.ts';
  import { ENTITY_TYPE_HEX_COLORS, RELATION_TYPE_COLORS } from '../api/types.ts';

  let container: HTMLDivElement | undefined = $state();
  let cy: Core | null = null;
  let readyFailed = $state(false);

  let lastVersion = 0;

  onMount(async () => {
    const result = await waitForReady();
    if (result === 'timeout') {
      readyFailed = true;
      return;
    }
    loadFullGraph();
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function buildStyles(): any[] {
    return [
      {
        selector: 'node',
        style: {
          'label': 'data(label)',
          'text-wrap': 'ellipsis' as const,
          'text-max-width': '80px',
          'font-size': '10px',
          'color': '#e1e2ec',
          'text-outline-color': '#0b0e15',
          'text-outline-width': 2,
          'background-color': 'data(type)',
          'width': 24,
          'height': 24,
          'border-width': 2,
          'border-color': '#fff',
          'border-opacity': 0.3,
        },
      },
      ...Object.entries(ENTITY_TYPE_HEX_COLORS).map(([type, color]) => ({
        selector: `node[type="${type}"]`,
        style: { 'background-color': color },
      })),
      {
        selector: 'edge',
        style: {
          'width': 1,
          'line-color': '#424754',
          'curve-style': 'bezier' as const,
          'opacity': 0.4,
          'label': 'data(label)',
          'font-size': '8px',
          'color': '#8b949e',
          'text-rotation': 'autorotate' as const,
          'text-opacity': 0.6,
        },
      },
      ...Object.entries(RELATION_TYPE_COLORS).map(([rel, color]) => ({
        selector: `edge[label="${rel}"]`,
        style: { 'line-color': color },
      })),
      {
        selector: 'node:selected',
        style: {
          'border-width': 3,
          'border-color': '#82b1ff',
          'border-opacity': 1,
        },
      },
    ];
  }

  $effect(() => {
    const data = getGraphData();
    const currentVersion = getVersion();
    if (!data || !container) return;

    // Skip recreation if data hasn't changed
    if (currentVersion === lastVersion && cy) return;
    lastVersion = currentVersion;

    if (cy) {
      // Incremental update: replace elements without full destroy
      cy.elements().remove();
      cy.add([
        ...data.nodes.map((n) => ({
          data: {
            id: n.data.id,
            label: n.data.label,
            type: n.data.type,
            category: n.data.category,
          },
        })),
        ...data.edges.map((e) => ({
          data: {
            id: e.data.id,
            source: e.data.source,
            target: e.data.target,
            label: e.data.label,
          },
        })),
      ]);
      cy.layout({
        name: 'cose',
        animate: true,
        animationDuration: 800,
        nodeRepulsion: 8000,
        idealEdgeLength: 100,
        gravity: 0.3,
      }).run();
      return;
    }

    cy = cytoscape({
      container,
      elements: [
        ...data.nodes.map((n) => ({
          data: {
            id: n.data.id,
            label: n.data.label,
            type: n.data.type,
            category: n.data.category,
          },
        })),
        ...data.edges.map((e) => ({
          data: {
            id: e.data.id,
            source: e.data.source,
            target: e.data.target,
            label: e.data.label,
          },
        })),
      ],
      style: buildStyles(),
      layout: {
        name: 'cose',
        animate: true,
        animationDuration: 800,
        nodeRepulsion: 8000,
        idealEdgeLength: 100,
        gravity: 0.3,
      },
    });

    cy.on('tap', 'node', (evt) => {
      const node: NodeSingular = evt.target;
      selectEntity(node.id());
    });
  });

  onDestroy(() => {
    cy?.destroy();
    cy = null;
  });

  export function fit() {
    cy?.fit(undefined, 40);
  }

  export function zoomIn() {
    cy?.zoom(cy.zoom() * 1.3);
    cy?.center();
  }

  export function zoomOut() {
    cy?.zoom(cy.zoom() / 1.3);
    cy?.center();
  }

  export function runLayout(name: string) {
    cy?.layout({ name: name as 'cose' | 'breadthfirst' | 'circle' | 'concentric', animate: true }).run();
  }
</script>

<div bind:this={container} class="w-full h-full">
  {#if readyFailed}
    <div class="flex items-center justify-center h-full">
      <div class="text-center space-y-2">
        <span class="material-symbols-outlined text-3xl text-[var(--color-error)]">cloud_off</span>
        <p class="text-sm text-[var(--color-error)]">Backend did not become ready in time</p>
        <button onclick={() => { readyFailed = false; loadFullGraph(); }}
          class="text-xs text-[var(--color-primary)] underline hover:no-underline">Retry</button>
      </div>
    </div>
  {:else if isLoading() && !getGraphData()}
    <div class="flex items-center justify-center h-full">
      <p class="text-[var(--color-on-surface-variant)] text-sm">Loading graph...</p>
    </div>
  {/if}
  {#if getError()}
    <div class="absolute bottom-4 left-4 right-4 z-50">
      <div class="glass-panel p-3 border-[var(--color-error)]/30 text-xs text-[var(--color-error)]">
        {getError()}
      </div>
    </div>
  {/if}
</div>
