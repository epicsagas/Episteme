import type { GraphStats } from '../api/types.ts';
import { stats as fetchStats } from '../api/endpoints.ts';
import { getBaseUrl } from './connection.svelte.ts';

let data: GraphStats | null = $state(null);
let loading = $state(false);
let error: string | null = $state(null);

export function getStats(): GraphStats | null {
  return data;
}

export function isLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return error;
}

export async function loadStats(): Promise<void> {
  loading = true;
  error = null;
  try {
    data = await fetchStats(getBaseUrl());
  } catch (e) {
    error = e instanceof Error ? e.message : 'Failed to load stats';
  } finally {
    loading = false;
  }
}
