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
    link_provenance?: string[];
    [key: string]: string[] | undefined;
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

export const ENTITY_TYPE_HEX_COLORS: Record<EntityType, string> = {
  pattern: '#10b981',
  refactoring: '#06b6d4',
  law: '#f59e0b',
  smell: '#ef4444',
  insight: '#6366f1',
};

export const ENTITY_TYPE_ICONS: Record<EntityType, string> = {
  pattern: 'design_services',
  refactoring: 'build',
  law: 'gavel',
  smell: 'warning',
  insight: 'lightbulb',
}

// Schema types for dynamic ontology loading

export interface SchemaEntityType {
  key: string;
  count: number;
}

export interface SchemaRelationType {
  key: string;
  inverse: string | null;
}

export interface SchemaResponse {
  entity_types: SchemaEntityType[];
  relation_types: SchemaRelationType[];
}

export const RELATION_TYPE_COLORS: Record<string, string> = {
  solves: '#34d399',
  solved_by: '#6ee7b7',
  enforces: '#22d3ee',
  enforced_by: '#67e8f9',
  violates: '#f87171',
  violated_by: '#fca5a5',
  related_to: '#78909c',
  derives_from: '#a78bfa',
  applies_to: '#2dd4bf',
  supersedes: '#fb923c',
};

export const RELATION_DESCRIPTIONS: Record<string, string> = {
  solves: 'Pattern/Refactoring solves a Smell',
  solved_by: 'Smell is solved by a Pattern/Refactoring',
  enforces: 'Pattern enforces a Law',
  enforced_by: 'Law is enforced by a Pattern',
  violates: 'Pattern violates a Law',
  violated_by: 'Law is violated by a Smell/Anti-pattern',
  related_to: 'General relationship',
  derives_from: 'Derived from another concept',
  applies_to: 'Applies to a context',
  supersedes: 'Supersedes an older concept',
};

export const DATA_SOURCES: Array<{ name: string; icon: string; entityType: EntityType }> = [
  { name: 'GoF Design Patterns', icon: 'menu_book', entityType: 'pattern' },
  { name: 'Refactoring Catalog (Fowler)', icon: 'auto_fix_high', entityType: 'refactoring' },
  { name: 'Software Laws & Principles', icon: 'gavel', entityType: 'law' },
  { name: 'Code Smells Catalog', icon: 'warning', entityType: 'smell' },
  { name: 'Tacit Knowledge Insights', icon: 'lightbulb', entityType: 'insight' },
];

// Insight creation types

export interface CreateInsightRequest {
  text: string;
  tags?: string[];
  linked_entities?: string[];
}

export interface InsightAutoLink {
  entity_id: string;
  score: string;
  link_type: 'auto' | 'suggested' | 'manual';
}

export interface CreateInsightResponse {
  id: string;
  auto_links: InsightAutoLink[];
  suggested_links: InsightAutoLink[];
  related_insights: Array<{ insight_id: string; combined: string }>;
  duplicates: Array<{ insight_id: string; overlap: string; note: string }>;
  confidence: number;
  error?: string;
};
