<script lang="ts">
  import { loadStats, getStats } from '../stores/stats.svelte.ts';
  import { getStatus } from '../stores/connection.svelte.ts';
  import { getWebUrl } from '../stores/connection.svelte.ts';
  import { navigate } from '../router/index.svelte.ts';
  import { ENTITY_TYPE_COLORS } from '../api/types.ts';

  let stats = $derived(getStats());
  let status = $derived(getStatus());

  type InsightItem = {
    id: string;
    title: string;
    description: string;
    project: string;
  };

  let insights: InsightItem[] = $state([]);
  let searchQuery = $state('');
  let selectedProject = $state('all');
  let loadingInsights = $state(false);

  $effect(() => {
    if (status === 'connected' && !stats) loadStats();
  });

  $effect(() => {
    if (status === 'connected' && insights.length === 0) loadInsights();
  });

  async function loadInsights() {
    loadingInsights = true;
    try {
      const res = await fetch(`${getWebUrl()}/api/graph/tree`);
      const data = await res.json();
      const insightTree = (data.tree ?? []).find((t: { type: string }) => t.type === 'insight');
      if (!insightTree) return;
      const items: InsightItem[] = [];
      for (const cat of insightTree.children ?? []) {
        for (const entity of cat.children ?? []) {
          const project = extractProject(entity.description ?? entity.title ?? '');
          items.push({
            id: entity.id,
            title: entity.title,
            description: entity.description ?? '',
            project,
          });
        }
      }
      insights = items;
    } catch {
      // silent
    } finally {
      loadingInsights = false;
    }
  }

  function extractProject(text: string): string {
    const m = text.match(/`([^`]+)`/);
    return m ? m[1] : 'unknown';
  }

  let projects = $derived([...new Set(insights.map(i => i.project))].sort());

  let filtered = $derived(
    insights.filter(i => {
      const matchProject = selectedProject === 'all' || i.project === selectedProject;
      const q = searchQuery.trim().toLowerCase();
      const matchSearch = !q || i.title.toLowerCase().includes(q) || i.description.toLowerCase().includes(q);
      return matchProject && matchSearch;
    })
  );

  function handleOpen(id: string) {
    navigate({ page: 'entity', id, from: 'dashboard' });
  }

  function formatScore(desc: string): string | null {
    const m = desc.match(/avg_score=([\d.]+)/);
    return m ? parseFloat(m[1]).toFixed(3) : null;
  }

  function formatSuccessRate(desc: string): string | null {
    const m = desc.match(/([\d.]+)%\s+success/);
    return m ? m[1] + '%' : null;
  }

  function formatObs(desc: string): string | null {
    const m = desc.match(/([\d,]+)\s+observations/);
    return m ? m[1] : null;
  }
</script>

<div class="space-y-5">
  <!-- Header + stats strip -->
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-bold text-[var(--color-on-surface)]">Insights</h2>
        <p class="text-xs text-[var(--color-on-surface-variant)] mt-0.5">
          TK (Tacit Knowledge) — 세션 성능 기록 자동 수집
        </p>
      </div>
      {#if stats}
        <div class="flex items-center gap-4 text-xs text-[var(--color-on-surface-variant)]">
          <span><span class="font-bold text-[var(--color-insight)]">{stats.by_type?.insight ?? 0}</span> insights</span>
          <span><span class="font-bold text-[var(--color-on-surface)]">{stats.total_entities}</span> total entities</span>
          <span><span class="font-bold text-[var(--color-secondary)]">{stats.total_edges}</span> relations</span>
        </div>
      {/if}
    </div>

    <!-- Search + filter bar -->
    <div class="flex gap-3">
      <div class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-surface-container)] border border-[var(--color-outline-variant)]/50">
        <span class="material-symbols-outlined text-sm text-[var(--color-on-surface-variant)]">search</span>
        <input
          type="text"
          placeholder="Search insights..."
          bind:value={searchQuery}
          class="flex-1 bg-transparent text-sm text-[var(--color-on-surface)] outline-none placeholder:text-[var(--color-on-surface-variant)]"
        />
        {#if searchQuery}
          <button onclick={() => searchQuery = ''} class="text-[var(--color-on-surface-variant)] hover:text-[var(--color-on-surface)]">
            <span class="material-symbols-outlined text-sm">close</span>
          </button>
        {/if}
      </div>
      <select
        bind:value={selectedProject}
        class="px-3 py-2 rounded-lg bg-[var(--color-surface-container)] border border-[var(--color-outline-variant)]/50 text-sm text-[var(--color-on-surface)] outline-none"
      >
        <option value="all">All projects</option>
        {#each projects as p}
          <option value={p}>{p}</option>
        {/each}
      </select>
    </div>

    {#if !loadingInsights}
      <p class="text-xs text-[var(--color-on-surface-variant)]">
        {filtered.length}개 표시 / 전체 {insights.length}개
      </p>
    {/if}
  </div>

  <!-- Insight list -->
  {#if loadingInsights}
    <div class="space-y-2">
      {#each Array(8) as _}
        <div class="h-16 rounded-lg bg-[var(--color-surface-container)] animate-pulse"></div>
      {/each}
    </div>
  {:else if filtered.length === 0}
    <div class="glass-panel p-10 flex flex-col items-center gap-3 text-center">
      <span class="material-symbols-outlined text-3xl text-[var(--color-on-surface-variant)]">
        {insights.length === 0 ? 'cloud_off' : 'search_off'}
      </span>
      <p class="text-sm text-[var(--color-on-surface-variant)]">
        {insights.length === 0 ? 'Insight 데이터를 불러올 수 없습니다' : '검색 결과 없음'}
      </p>
    </div>
  {:else}
    <div class="space-y-1.5">
      {#each filtered as item}
        <button
          onclick={() => handleOpen(item.id)}
          class="w-full glass-panel px-4 py-3 flex items-start gap-3 hover:border-[var(--color-insight)]/40 transition-all text-left group"
        >
          <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 mt-0.5"
            style="background: color-mix(in srgb, {ENTITY_TYPE_COLORS['insight']} 15%, transparent);">
            <span class="material-symbols-outlined text-sm" style="color: {ENTITY_TYPE_COLORS['insight']}">lightbulb</span>
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="font-mono text-[10px] text-[var(--color-on-surface-variant)]">{item.id}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-surface-container-high)] text-[var(--color-on-surface-variant)]">
                {item.project}
              </span>
            </div>
            <p class="text-sm text-[var(--color-on-surface)] truncate mt-0.5">{item.description}</p>
          </div>

          <div class="flex items-center gap-3 shrink-0 text-right">
            {#if formatSuccessRate(item.description)}
              <div class="text-right">
                <p class="text-xs font-bold text-[var(--color-rel-solves)]">{formatSuccessRate(item.description)}</p>
                <p class="text-[10px] text-[var(--color-on-surface-variant)]">success</p>
              </div>
            {/if}
            {#if formatScore(item.description)}
              <div class="text-right">
                <p class="text-xs font-bold text-[var(--color-insight)]">{formatScore(item.description)}</p>
                <p class="text-[10px] text-[var(--color-on-surface-variant)]">avg score</p>
              </div>
            {/if}
            {#if formatObs(item.description)}
              <div class="text-right hidden sm:block">
                <p class="text-xs font-bold text-[var(--color-on-surface)]">{formatObs(item.description)}</p>
                <p class="text-[10px] text-[var(--color-on-surface-variant)]">obs</p>
              </div>
            {/if}
            <span class="material-symbols-outlined text-sm text-[var(--color-outline)] group-hover:text-[var(--color-insight)] transition-colors">chevron_right</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
