import type { Entity, CytoscapeGraph } from '../api/types.ts';
import { getEntity, getFullGraph, getNeighbors } from '../api/endpoints.ts';
import { getBaseUrl, getWebUrl } from './connection.svelte.ts';
import type { Neighbor } from '../api/types.ts';

let selectedEntity: Entity | null = $state(null);
let neighbors: Neighbor[] = $state([]);
let graphData: CytoscapeGraph | null = $state(null);
let version: number = $state(0);
let loading = $state(false);
let errorMsg: string | null = $state(null);

export function getSelectedEntity(): Entity | null {
  return selectedEntity;
}

export function getNeighborsList(): Neighbor[] {
  return neighbors;
}

export function getGraphData(): CytoscapeGraph | null {
  return graphData;
}

export function getVersion(): number {
  return version;
}

export function isLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return errorMsg;
}

export async function loadFullGraph(): Promise<void> {
  loading = true;
  errorMsg = null;
  try {
    graphData = await getFullGraph(getWebUrl());
    version++;
  } catch {
    errorMsg = 'Failed to load graph data';
  } finally {
    loading = false;
  }
}

export async function selectEntity(id: string): Promise<void> {
  loading = true;
  errorMsg = null;
  try {
    const [entity, neighborhood] = await Promise.all([
      getEntity(getBaseUrl(), id),
      getNeighbors(getBaseUrl(), id),
    ]);
    selectedEntity = entity;
    neighbors = neighborhood.neighbors;
  } catch (e) {
    errorMsg = e instanceof Error ? e.message : 'Failed to load entity';
  } finally {
    loading = false;
  }
}

export function clearSelection() {
  selectedEntity = null;
  neighbors = [];
}
