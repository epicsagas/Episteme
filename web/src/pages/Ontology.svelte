<script lang="ts">
  import { onMount } from 'svelte';
  import { getTree, getSchema } from '../api/endpoints.ts';
  import { getWebUrl } from '../stores/connection.svelte.ts';
  import Badge from '../ui/Badge.svelte';
  import Skeleton from '../ui/Skeleton.svelte';
  import EmptyState from '../ui/EmptyState.svelte';
  import { ENTITY_TYPE_ICONS, ENTITY_TYPE_COLORS, ENTITY_TYPE_LABELS, RELATION_TYPE_COLORS, RELATION_DESCRIPTIONS, DATA_SOURCES, ENTITY_TYPE_HEX_COLORS } from '../api/types.ts';
  import type { TreeNode, EntityType, SchemaRelationType } from '../api/types.ts';
  import { navigate } from '../router/index.svelte.ts';

  let trees: TreeNode[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let expandedTypes = $state<Set<string>>(new Set());
  let relationTypes: SchemaRelationType[] = $state([]);
  let schemaEntityCounts: Map<string, number> = $state(new Map());

  onMount(async () => {
    try {
      const webUrl = getWebUrl();
      const [treeRes, schema] = await Promise.all([
        getTree(webUrl),
        getSchema(webUrl),
      ]);
      trees = treeRes.tree;
      relationTypes = schema.relation_types;
      schemaEntityCounts = new Map(schema.entity_types.map(e => [e.key, e.count]));
      expandedTypes = new Set(trees.map(t => t.type));
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load ontology';
    } finally {
      loading = false;
    }
  });

  function toggleType(type: string) {
    const next = new Set(expandedTypes);
    if (next.has(type)) {
      next.delete(type);
    } else {
      next.add(type);
    }
    expandedTypes = next;
  }

  function handleNavigate(id: string) {
    navigate({ page: 'entity', id });
  }
</script>

<div class="space-y-6">
  <section class="flex items-end justify-between border-b border-[var(--color-outline-variant)] pb-6">
    <div>
      <h1 class="text-3xl font-bold tracking-tight text-[var(--color-on-surface)]">Ontology</h1>
      <p class="text-sm text-[var(--color-on-surface-variant)] mt-1">Schema, categories, and data sources of the knowledge graph</p>
    </div>
  </section>

  {#if loading}
    <div class="space-y-6">
      {#each [0, 1, 2] as _}
        <Skeleton class="h-48" />
      {/each}
    </div>
  {:else if error}
    <EmptyState icon="error" message={error} />
  {:else}
    <div class="grid grid-cols-12 gap-6">
      <!-- Schema: Entity Types -->
      <div class="col-span-12 lg:col-span-8 space-y-4">
        <h2 class="text-lg font-bold text-[var(--color-on-surface)] flex items-center gap-2">
          <span class="material-symbols-outlined text-[var(--color-primary)]">schema</span>
          Entity Types
        </h2>
        {#each trees as tree}
          <div class="glass-panel overflow-hidden">
            <button
              onclick={() => toggleType(tree.type)}
              class="w-full flex items-center justify-between p-5 hover:bg-[var(--color-surface-container-high)]/30 transition-colors"
            >
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg flex items-center justify-center border"
                  style="background: color-mix(in srgb, {ENTITY_TYPE_COLORS[tree.type as EntityType]} 15%, transparent);
                         border-color: color-mix(in srgb, {ENTITY_TYPE_COLORS[tree.type as EntityType]} 30%, transparent);">
                  <span class="material-symbols-outlined text-sm"
                    style="color: {ENTITY_TYPE_COLORS[tree.type as EntityType]}">{ENTITY_TYPE_ICONS[tree.type as EntityType]}</span>
                </div>
                <div class="text-left">
                  <p class="font-bold text-[var(--color-on-surface)]">{ENTITY_TYPE_LABELS[tree.type as EntityType] || tree.label}</p>
                  <p class="text-xs text-[var(--color-on-surface-variant)]">{tree.children.length} categories</p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <Badge type={tree.type as EntityType} small />
                <span class="material-symbols-outlined text-[var(--color-outline)] transition-transform"
                  style="transform: rotate({expandedTypes.has(tree.type) ? '180deg' : '0deg'})">expand_more</span>
              </div>
            </button>

            {#if expandedTypes.has(tree.type)}
              <div class="border-t border-[var(--color-outline-variant)]/30">
                {#each tree.children as category, ci}
                  <div class="border-b border-[var(--color-outline-variant)]/20 last:border-b-0">
                    <div class="px-5 py-3 bg-[var(--color-surface-container-low)]/50">
                      <p class="text-xs font-bold uppercase tracking-wider text-[var(--color-on-surface-variant)]">{category.label || category.category}</p>
                    </div>
                    <div class="px-5 py-2">
                      {#each category.children as entity}
                        <button
                          onclick={() => handleNavigate(entity.id)}
                          class="w-full flex items-center gap-3 py-2 px-2 rounded hover:bg-[var(--color-surface-container-high)]/20 transition-colors cursor-pointer text-left"
                        >
                          <span class="material-symbols-outlined text-sm"
                            style="color: {ENTITY_TYPE_COLORS[tree.type as EntityType]}">{ENTITY_TYPE_ICONS[tree.type as EntityType]}</span>
                          <div class="flex-1 min-w-0">
                            <p class="text-sm text-[var(--color-on-surface)] truncate">{entity.title}</p>
                            {#if entity.description}
                              <p class="text-[11px] text-[var(--color-on-surface-variant)] truncate">{entity.description}</p>
                            {/if}
                          </div>
                          <span class="material-symbols-outlined text-xs text-[var(--color-outline)]">chevron_right</span>
                        </button>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Sidebar: Sources & Stats -->
      <div class="col-span-12 lg:col-span-4 space-y-6">
        <!-- Relation Types -->
        <div class="glass-panel p-6">
          <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-4">
            <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">link</span>
            Relation Types
          </h3>
          <div class="space-y-2">
            {#each relationTypes as rel}
              <div class="flex items-start gap-3 py-1.5">
                <div class="w-5 h-1 rounded-full mt-1.5 shrink-0" style="background: {RELATION_TYPE_COLORS[rel.key] || '#78909c'}"></div>
                <div>
                  <p class="text-xs font-mono font-bold text-[var(--color-on-surface)]">{rel.key}</p>
                  <p class="text-[10px] text-[var(--color-on-surface-variant)]">{RELATION_DESCRIPTIONS[rel.key] || ''}</p>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Data Sources -->
        <div class="glass-panel p-6">
          <h3 class="font-bold text-[var(--color-on-surface)] flex items-center gap-2 mb-4">
            <span class="material-symbols-outlined text-[var(--color-primary)] text-sm">source</span>
            Data Sources
          </h3>
          <div class="space-y-3">
            {#each DATA_SOURCES as source}
              <div class="flex items-center gap-3 py-1">
                <span class="material-symbols-outlined text-sm text-[var(--color-on-surface-variant)]">{source.icon}</span>
                <div class="flex-1 min-w-0">
                  <span class="text-sm text-[var(--color-on-surface)]">{source.name}</span>
                  <span class="text-[10px] text-[var(--color-on-surface-variant)] ml-1">({schemaEntityCounts.get(source.entityType) || 0})</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
