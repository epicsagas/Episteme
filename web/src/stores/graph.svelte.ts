import type { Entity, CytoscapeGraph } from '../api/types.ts';
import { getEntity, getFullGraph, getNeighbors } from '../api/endpoints.ts';
import { getBaseUrl, getWebUrl } from './connection.svelte.ts';
import type { Neighbor } from '../api/types.ts';

let selectedEntity: Entity | null = $state(null);
let neighbors: Neighbor[] = $state([]);
let graphData: CytoscapeGraph | null = $state(null);
let loading = $state(false);
let error: string | null = $state(null);

export function getSelectedEntity(): Entity | null {
  return selectedEntity;
}

export function getNeighborsList(): Neighbor[] {
  return neighbors;
}

export function getGraphData(): CytoscapeGraph | null {
  return graphData;
}

export function isLoading(): boolean {
  return loading;
}

export async function loadFullGraph(): Promise<void> {
  loading = true;
  error = null;
  try {
    graphData = await getFullGraph(getWebUrl());
  } catch {
    error = 'Failed to load graph data';
  } finally {
    loading = false;
  }
}

export async function selectEntity(id: string): Promise<void> {
  loading = true;
  error = null;
  try {
    const [entity, neighborhood] = await Promise.all([
      getEntity(getBaseUrl(), id),
      getNeighbors(getBaseUrl(), id),
    ]);
    selectedEntity = entity;
    neighbors = neighborhood.neighbors;
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to load entity';
  } finally {
    loading = false;
  }
}

export function clearSelection() {
  selectedEntity = null;
  neighbors = [];
}
