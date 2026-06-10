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
          'text-max-width': '90px',
          'font-size': '11px',
          'font-family': 'Inter, system-ui, sans-serif',
          'color': 'var(--color-on-surface)',
          'text-outline-color': 'var(--color-surface-container-lowest)',
          'text-outline-width': 3,
          'text-valign': 'bottom',
          'text-margin-y': 4,
          'background-color': 'data(type)',
          'width': 32,
          'height': 32,
          'shape': 'ellipse',
          'border-width': 2,
          'border-color': 'data(type)',
          'border-opacity': 0.6,
          'font-weight': 500,
        },
      },
      // Per-type node styles with 15% fill + 100% border
      ...Object.entries(ENTITY_TYPE_HEX_COLORS).map(([type, color]) => ({
        selector: `node[type="${type}"]`,
        style: {
          'background-color': color,
          'background-opacity': 0.2,
          'border-color': color,
          'border-opacity': 0.9,
        },
      })),
      {
        selector: 'edge',
        style: {
          'width': 1.5,
          'line-color': 'var(--color-outline-variant)',
          'curve-style': 'bezier' as const,
          'opacity': 0.35,
          'label': 'data(label)',
          'font-size': '9px',
          'font-family': 'Inter, system-ui, sans-serif',
          'color': 'var(--color-on-surface-variant)',
          'text-rotation': 'autorotate' as const,
          'text-opacity': 0,
          'text-background-color': 'var(--color-surface-container-lowest)',
          'text-background-opacity': 0.85,
          'text-background-padding': '3px',
        },
      },
      // Show edge labels on hover
      {
        selector: 'edge.hovered',
        style: {
          'text-opacity': 0.9,
          'opacity': 0.7,
          'width': 2,
        },
      },
      // Per-relation edge colors
      ...Object.entries(RELATION_TYPE_COLORS).map(([rel, color]) => ({
        selector: `edge[label="${rel}"]`,
        style: {
          'line-color': color,
          'line-opacity': 0.5,
        },
      })),
      {
        selector: 'node:selected',
        style: {
          'border-width': 3,
          'border-color': 'var(--color-primary)',
          'border-opacity': 1,
          'background-opacity': 0.35,
        },
      },
      {
        selector: 'node.hovered',
        style: {
          'border-width': 3,
          'border-opacity': 1,
          'background-opacity': 0.3,
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

    const elements = [
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
    ];

    if (cy) {
      cy.elements().remove();
      cy.add(elements);
      cy.layout({
        name: 'cose',
        animate: true,
        animationDuration: 600,
        nodeRepulsion: 10000,
        idealEdgeLength: 120,
        gravity: 0.25,
      }).run();
      return;
    }

    cy = cytoscape({
      container,
      elements,
      style: buildStyles(),
      layout: {
        name: 'cose',
        animate: true,
        animationDuration: 800,
        nodeRepulsion: 10000,
        idealEdgeLength: 120,
        gravity: 0.25,
      },
      minZoom: 0.15,
      maxZoom: 4,
    });

    // Node click → select
    cy.on('tap', 'node', (evt) => {
      const node: NodeSingular = evt.target;
      selectEntity(node.id());
    });

    // Edge hover → show label
    cy.on('mouseover', 'edge', (evt) => {
      evt.target.addClass('hovered');
    });
    cy.on('mouseout', 'edge', (evt) => {
      evt.target.removeClass('hovered');
    });

    // Node hover → highlight
    cy.on('mouseover', 'node', (evt) => {
      evt.target.addClass('hovered');
      document.body.style.cursor = 'pointer';
    });
    cy.on('mouseout', 'node', (evt) => {
      evt.target.removeClass('hovered');
      document.body.style.cursor = 'default';
    });

    // LOD: hide labels/edges when zoomed out
    cy.on('zoom', () => {
      if (!cy) return;
      const z = cy.zoom();
      const nodes = cy.nodes();
      const edges = cy.edges();

      if (z < 0.3) {
        nodes.style({ 'label': '', 'opacity': 0.5 });
        edges.style({ 'opacity': 0.1 });
      } else if (z < 0.6) {
        nodes.style({ 'label': '', 'opacity': 0.8 });
        edges.style({ 'opacity': 0.25 });
      } else {
        nodes.style({ 'label': 'data(label)' });
        nodes.removeStyle('opacity');
        edges.removeStyle('opacity');
      }
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
    cy?.zoom(cy!.zoom() * 1.3);
    cy?.center();
  }

  export function zoomOut() {
    cy?.zoom(cy!.zoom() / 1.3);
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
      <div class="flex items-center gap-2">
        <span class="material-symbols-outlined text-[var(--color-on-surface-variant)] animate-spin text-xl">progress_activity</span>
        <p class="text-[var(--color-on-surface-variant)] text-sm">Loading graph...</p>
      </div>
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
