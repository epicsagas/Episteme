<script lang="ts">
  import cytoscape from 'cytoscape';
  import type { Core, NodeSingular } from 'cytoscape';
  import { onMount, onDestroy } from 'svelte';
  import { loadFullGraph, getGraphData, isLoading, getError, getVersion } from '../stores/graph.svelte.ts';
  import { selectEntity } from '../stores/graph.svelte.ts';
  import { waitForReady, checkHealth } from '../stores/connection.svelte.ts';
  import { ENTITY_TYPE_HEX_COLORS, RELATION_TYPE_COLORS } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';
  import { isDark } from '../stores/theme.svelte.ts';

  let container: HTMLDivElement | undefined = $state();
  let cy: Core | null = null;
  let readyFailed = $state(false);
  let showInsights = $state(false);
  let showIsolated = $state(false);

  let lastVersion = 0;

  // Entity type → shape mapping for visual distinction
  const ENTITY_SHAPES: Record<EntityType, string> = {
    pattern: 'round-rectangle',
    refactoring: 'diamond',
    law: 'hexagon',
    smell: 'triangle',
    insight: 'ellipse',
  };

  // Node size by entity type (visual hierarchy)
  const ENTITY_SIZES: Record<EntityType, number> = {
    pattern: 40,
    law: 38,
    refactoring: 34,
    smell: 32,
    insight: 28,
  };

  onMount(async () => {
    const result = await waitForReady();
    if (result === 'timeout') {
      readyFailed = true;
      return;
    }
    loadFullGraph();
  });

  function getThemeColors() {
    const dark = isDark();
    return {
      textColor: dark ? '#c2c6d6' : '#43474e',
      textBg: dark ? 'rgba(11, 14, 21, 0.92)' : 'rgba(255, 255, 255, 0.92)',
      edgeColor: dark ? 'rgba(48, 54, 61, 0.6)' : 'rgba(196, 198, 207, 0.5)',
      edgeLabelColor: dark ? '#8c909f' : '#74777f',
      selectionBorder: dark ? '#adc6ff' : '#002045',
      nodeFillOpacity: dark ? 0.25 : 0.18,
      nodeBorderOpacity: dark ? 0.95 : 0.85,
      edgeLineOpacity: dark ? 0.45 : 0.35,
    };
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function buildStyles(): any[] {
    const c = getThemeColors();

    return [
      {
        selector: 'node',
        style: {
          'label': 'data(label)',
          'text-wrap': 'ellipsis' as const,
          'text-max-width': '80px',
          'font-size': '10px',
          'font-family': 'Inter, system-ui, sans-serif',
          'color': c.textColor,
          'text-outline-color': c.textBg,
          'text-outline-width': 3,
          'text-valign': 'bottom',
          'text-margin-y': 4,
          'background-color': 'data(type)',
          'width': 32,
          'height': 32,
          'border-width': 2,
          'border-color': 'data(type)',
          'border-opacity': 0.7,
          'font-weight': 500,
        },
      },
      // Per-type node styles with shapes, sizes, and colors
      ...(Object.entries(ENTITY_TYPE_HEX_COLORS) as [EntityType, string][]).map(([type, color]) => ({
        selector: `node[type="${type}"]`,
        style: {
          'background-color': color,
          'background-opacity': c.nodeFillOpacity,
          'border-color': color,
          'border-opacity': c.nodeBorderOpacity,
          'shape': ENTITY_SHAPES[type],
          'width': ENTITY_SIZES[type],
          'height': ENTITY_SIZES[type],
        },
      })),
      {
        selector: 'edge',
        style: {
          'width': 1.2,
          'line-color': c.edgeColor,
          'curve-style': 'bezier' as const,
          'opacity': 0.4,
          'label': 'data(label)',
          'font-size': '8px',
          'font-family': 'Inter, system-ui, sans-serif',
          'color': c.edgeLabelColor,
          'text-rotation': 'autorotate' as const,
          'text-opacity': 0,
          'text-background-color': c.textBg,
          'text-background-opacity': 0.9,
          'text-background-padding': '3px',
        },
      },
      // Show edge labels on hover
      {
        selector: 'edge.hovered',
        style: {
          'text-opacity': 0.95,
          'opacity': 0.75,
          'width': 2.5,
        },
      },
      // Per-relation edge colors
      ...(Object.entries(RELATION_TYPE_COLORS) as [string, string][]).map(([rel, color]) => ({
        selector: `edge[label="${rel}"]`,
        style: {
          'line-color': color,
          'line-opacity': c.edgeLineOpacity,
        },
      })),
      {
        selector: 'node:selected',
        style: {
          'border-width': 3.5,
          'border-color': c.selectionBorder,
          'border-opacity': 1,
          'background-opacity': 0.4,
        },
      },
      {
        selector: 'node.hovered',
        style: {
          'border-width': 3,
          'border-opacity': 1,
          'background-opacity': 0.35,
        },
      },
    ];
  }

  // Rebuild cytoscape instance when graph data changes
  $effect(() => {
    const data = getGraphData();
    const currentVersion = getVersion();
    if (!data || !container) return;

    // Skip if data and insight filter haven't changed
    const effectiveVersion = currentVersion * 4 + (showInsights ? 2 : 0) + (showIsolated ? 1 : 0);
    if (effectiveVersion === lastVersion && cy) return;
    lastVersion = effectiveVersion;

    // Destroy existing instance and recreate cleanly
    if (cy) {
      cy.destroy();
      cy = null;
    }

    // Build edge set to detect isolated nodes
    const allEdgeSources = new Set(data.edges.map(e => e.data.source));
    const allEdgeTargets = new Set(data.edges.map(e => e.data.target));
    const connectedIds = new Set([...allEdgeSources, ...allEdgeTargets]);

    const visibleNodes = data.nodes.filter(n => {
      if (!showInsights && n.data.type === 'insight') return false;
      if (!showIsolated && !connectedIds.has(n.data.id)) return false;
      return true;
    });
    const visibleIds = new Set(visibleNodes.map(n => n.data.id));
    const elements = [
      ...visibleNodes.map((n) => ({
        data: {
          id: n.data.id,
          label: n.data.label,
          type: n.data.type,
          category: n.data.category,
        },
      })),
      ...data.edges.filter(e => visibleIds.has(e.data.source) && visibleIds.has(e.data.target)).map((e) => ({
        data: {
          id: e.data.id,
          source: e.data.source,
          target: e.data.target,
          label: e.data.label,
        },
      })),
    ];

    cy = cytoscape({
      container,
      elements,
      style: buildStyles(),
      layout: {
        name: 'cose',
        animate: false,
        nodeRepulsion: 3500,
        idealEdgeLength: 90,
        gravity: 2.5,
        gravityRange: 5.0,
        nestingFactor: 1.5,
        componentSpacing: 20,
        numIter: 1500,
        coolingFactor: 0.99,
        minTemp: 1.0,
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
        nodes.removeStyle('label');
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
    if (cy) {
      cy.zoom(cy.zoom() * 1.3);
      cy.center();
    }
  }

  export function zoomOut() {
    if (cy) {
      cy.zoom(cy.zoom() / 1.3);
      cy.center();
    }
  }

  export function runLayout(name: string) {
    cy?.layout({ name: name as 'cose' | 'breadthfirst' | 'circle' | 'concentric', animate: false }).run();
  }

  export function setShowInsights(value: boolean) {
    showInsights = value;
  }

  export function setShowIsolated(value: boolean) {
    showIsolated = value;
  }
</script>

<div bind:this={container} class="w-full h-full">
  {#if readyFailed}
    <div class="flex items-center justify-center h-full">
      <div class="text-center space-y-3">
        <span class="material-symbols-outlined text-4xl text-[var(--color-error)]">cloud_off</span>
        <p class="text-sm text-[var(--color-error)]">Backend server is not reachable</p>
        <button onclick={() => { readyFailed = false; checkHealth().then(ok => { if (ok) loadFullGraph(); }); }}
          class="px-4 py-1.5 text-xs rounded-lg border border-[var(--color-primary)] text-[var(--color-primary)]
            hover:bg-[var(--color-primary)] hover:text-[var(--color-on-primary)] transition-colors">Retry</button>
      </div>
    </div>
  {:else if isLoading() && !getGraphData()}
    <div class="flex items-center justify-center h-full">
      <div class="flex items-center gap-3">
        <span class="material-symbols-outlined text-[var(--color-primary)] animate-spin text-xl">progress_activity</span>
        <p class="text-[var(--color-on-surface-variant)] text-sm">Loading knowledge graph...</p>
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
