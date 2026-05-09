import { apiGet, apiPost } from './client.ts';
import type {
  GraphStats,
  Entity,
  EntitySummary,
  SearchResponse,
  Neighborhood,
  PathResult,
  Subgraph,
  HealthResponse,
  CytoscapeGraph,
  TreeNode,
  SankeyData,
  EntityType,
} from './types.ts';

export function health(baseUrl: string): Promise<HealthResponse> {
  return apiGet<HealthResponse>(baseUrl, '/health');
}

export function stats(baseUrl: string): Promise<GraphStats> {
  return apiGet<GraphStats>(baseUrl, '/stats');
}

export function search(baseUrl: string, query: string, limit = 10, entityType?: EntityType): Promise<SearchResponse> {
  return apiPost<{ query: string; limit: number; entity_type?: EntityType }, SearchResponse>(
    baseUrl, '/search', { query, limit, entity_type: entityType },
  );
}

export function getEntity(baseUrl: string, id: string, detail: 'minimal' | 'summary' | 'detailed' | 'full' = 'full'): Promise<Entity> {
  return apiGet<Entity>(baseUrl, `/graph/${id}?detail=${detail}`);
}

export function getNeighbors(baseUrl: string, id: string, type?: string): Promise<Neighborhood> {
  const path = type ? `/graph/${id}/neighbors?type=${type}` : `/graph/${id}/neighbors`;
  return apiGet<Neighborhood>(baseUrl, path);
}

export function findPath(baseUrl: string, fromId: string, toId: string, maxDepth = 5): Promise<PathResult> {
  return apiPost<{ from_id: string; to_id: string; max_depth: number }, PathResult>(
    baseUrl, '/graph/path', { from_id: fromId, to_id: toId, max_depth: maxDepth },
  );
}

export function getSubgraph(baseUrl: string, entityId: string, depth = 2): Promise<Subgraph> {
  return apiPost<{ entity_id: string; depth: number }, Subgraph>(
    baseUrl, '/graph/subgraph', { entity_id: entityId, depth },
  );
}

export function getContradictions(baseUrl: string): Promise<Array<{ entity_id: string; title: string; conflicts: string[] }>> {
  return apiGet(baseUrl, '/graph/contradictions');
}

// Web Viewer API (port 8080)
export function getFullGraph(webUrl: string): Promise<CytoscapeGraph> {
  return apiGet<CytoscapeGraph>(webUrl, '/api/graph/full');
}

export function getTree(webUrl: string): Promise<{ tree: TreeNode[] }> {
  return apiGet<{ tree: TreeNode[] }>(webUrl, '/api/graph/tree');
}

export function getSankey(webUrl: string): Promise<SankeyData> {
  return apiGet<SankeyData>(webUrl, '/api/graph/sankey');
}
