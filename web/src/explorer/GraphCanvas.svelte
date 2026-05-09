<script lang="ts">
  import cytoscape from 'cytoscape';
  import type { Core, NodeSingular } from 'cytoscape';
  import { onMount, onDestroy } from 'svelte';
  import { loadFullGraph, getGraphData, isLoading } from '../stores/graph.svelte.ts';
  import { selectEntity } from '../stores/graph.svelte.ts';
  import {
    ENTITY_TYPE_COLORS,
    ENTITY_TYPE_ICONS,
  } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';

  let container: HTMLDivElement | undefined = $state();
  let cy: Core | null = null;

  const TYPE_COLORS: Record<string, string> = {
    pattern: '#4caf50',
    refactoring: '#2196f3',
    law: '#ff9800',
    smell: '#f44336',
    insight: '#ab47bc',
  };

  const REL_COLORS: Record<string, string> = {
    solves: '#66bb6a',
    solved_by: '#81c784',
    enforces: '#42a5f5',
    enforced_by: '#64b5f6',
    violates: '#ef5350',
    violated_by: '#e57373',
    related_to: '#78909c',
    derives_from: '#9575cd',
    applies_to: '#4db6ac',
    supersedes: '#ff8a65',
  };

  onMount(() => {
    loadFullGraph();
  });

  $effect(() => {
    const data = getGraphData();
    if (!data || !container) return;

    if (cy) {
      cy.destroy();
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
      style: [
        {
          selector: 'node',
          style: {
            'label': 'data(label)',
            'text-wrap': 'ellipsis',
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
        ...Object.entries(TYPE_COLORS).map(([type, color]) => ({
          selector: `node[type="${type}"]`,
          style: { 'background-color': color },
        })),
        {
          selector: 'edge',
          style: {
            'width': 1,
            'line-color': '#424754',
            'curve-style': 'bezier',
            'opacity': 0.4,
            'label': 'data(label)',
            'font-size': '8px',
            'color': '#8b949e',
            'text-rotation': 'autorotate',
            'text-opacity': 0.6,
          },
        },
        ...Object.entries(REL_COLORS).map(([rel, color]) => ({
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
      ],
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
  {#if isLoading() && !getGraphData()}
    <div class="flex items-center justify-center h-full">
      <p class="text-[var(--color-on-surface-variant)] text-sm">Loading graph...</p>
    </div>
  {/if}
</div>
