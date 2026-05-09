export type EntityType = 'pattern' | 'refactoring' | 'law' | 'smell' | 'insight';
export type RelationType = 'solves' | 'solved_by' | 'enforces' | 'enforced_by' | 'violates' | 'violated_by' | 'related_to' | 'derives_from' | 'applies_to' | 'supersedes';

export interface Entity {
  id: string;
  type: EntityType;
  title: string;
  description: string;
  name: string;
  category: string;
  tags: string[];
  relations: Partial<Record<RelationType, string[]>>;
  context: {
    benefits?: string[];
    when_to_use?: string[];
    drawbacks?: string[];
    consequences?: string[];
  };
  file_path: string;
  source: string | null;
}

export interface EntitySummary {
  id: string;
  title: string;
  type: EntityType;
  category: string;
  summary?: string;
}

export interface GraphStats {
  total_entities: number;
  total_edges: number;
  by_type: Record<EntityType, number>;
}

export interface SearchResult {
  entity_id: string;
  title: string;
  type: EntityType;
  category: string;
  score: string;
  section?: string;
  text?: string;
}

export interface SearchResponse {
  count: number;
  results: SearchResult[];
}

export interface Neighbor {
  id: string;
  title: string;
  type: EntityType;
}

export interface Neighborhood {
  entity_id: string;
  relation_type?: RelationType;
  neighbors: Neighbor[];
}

export interface PathResult {
  from: string;
  to: string;
  length: number;
  path: Array<{ id: string; title: string }>;
}

export interface GraphEdge {
  from_id: string;
  to_id: string;
  relation_type: RelationType;
}

export interface Subgraph {
  nodes: string[];
  edges: GraphEdge[];
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_secs: number;
  components: {
    knowledge_graph: string;
    rag_database: string;
    embedding_provider: string;
  };
}

export interface CytoscapeNode {
  data: {
    id: string;
    label: string;
    description?: string;
    type: EntityType;
    category?: string;
  };
}

export interface CytoscapeEdge {
  data: {
    id: string;
    source: string;
    target: string;
    label: RelationType;
  };
}

export interface CytoscapeGraph {
  nodes: CytoscapeNode[];
  edges: CytoscapeEdge[];
}

export interface TreeNode {
  type: EntityType;
  label: string;
  children: Array<{
    category: string;
    label: string;
    children: Array<{ id: string; title: string; description: string }>;
  }>;
}

export interface SankeyData {
  nodes: Array<{ id: string; label: string; count: number }>;
  links: Array<{ source: string; target: string; relation: string; value: number }>;
}

export const ENTITY_TYPE_LABELS: Record<EntityType, string> = {
  pattern: 'Design Patterns',
  refactoring: 'Refactorings',
  law: 'Laws & Principles',
  smell: 'Code Smells',
  insight: 'Insights',
};

export const ENTITY_TYPE_COLORS: Record<EntityType, string> = {
  pattern: 'var(--color-pattern)',
  refactoring: 'var(--color-refactoring)',
  law: 'var(--color-law)',
  smell: 'var(--color-smell)',
  insight: 'var(--color-insight)',
};

export const ENTITY_TYPE_ICONS: Record<EntityType, string> = {
  pattern: 'design_services',
  refactoring: 'build',
  law: 'gavel',
  smell: 'warning',
  insight: 'lightbulb',
};
