<script>
  import TrendChart from './lib/TrendChart.svelte';
  import LatencySummary from './lib/LatencySummary.svelte';
  import QueryTable from './lib/QueryTable.svelte';

  // Load all benchmark result files eagerly
  const rawFiles = import.meta.glob('/results/*.json', { eager: true });

  function parseTimestamp(filename) {
    // filename: search_benchmark_YYYYMMDD_HHMMSS.json
    const m = filename.match(/(\d{8})_(\d{6})\.json$/);
    if (!m) return new Date(0);
    const [, date, time] = m;
    const y = date.slice(0, 4);
    const mo = date.slice(4, 6);
    const d = date.slice(6, 8);
    const h = time.slice(0, 2);
    const mi = time.slice(2, 4);
    const s = time.slice(4, 6);
    return new Date(`${y}-${mo}-${d}T${h}:${mi}:${s}Z`);
  }

  // Build sorted runs array
  const runs = Object.entries(rawFiles)
    .map(([path, mod]) => {
      const filename = path.split('/').at(-1);
      const data = mod.default ?? mod;
      return {
        filename,
        path,
        timestamp: parseTimestamp(filename),
        label: filename.replace('search_benchmark_', '').replace('.json', ''),
        summary: data.summary,
        per_query: data.per_query ?? [],
      };
    })
    .sort((a, b) => a.timestamp - b.timestamp);

  // Trend data derived from all runs
  const trendRuns = runs.map((r) => ({
    label: r.label,
    timestamp: r.timestamp,
    hit1: r.summary.quality['hit@1'],
    mrr: r.summary.quality['mrr@5'],
    ndcg: r.summary.quality['ndcg@5'],
    queries: r.summary.queries,
    latencyMean: r.summary.latency_ms.mean,
    topK: r.summary.top_k,
  }));

  // Selected run index (default = latest)
  let selectedIndex = $state(runs.length - 1);

  let selectedRun = $derived(runs[selectedIndex]);
</script>

<div class="app">
  <header>
    <h1>Syntagma <span class="accent">Benchmark</span> Dashboard</h1>
    <p class="subtitle">{runs.length} runs loaded</p>
  </header>

  <!-- Section 1: Trend -->
  <section class="section">
    <h2>Trend</h2>
    <TrendChart runs={trendRuns} />
  </section>

  <!-- Section 2: Latest Run -->
  <section class="section">
    <div class="section-header">
      <h2>Run Details</h2>
      <select bind:value={selectedIndex} class="run-select">
        {#each runs as run, i}
          <option value={i}>{run.label}</option>
        {/each}
      </select>
    </div>
    {#if selectedRun}
      <LatencySummary summary={selectedRun.summary} />
    {/if}
  </section>

  <!-- Section 3: Per-Query -->
  <section class="section">
    <h2>Per-Query Results</h2>
    {#if selectedRun}
      <QueryTable perQuery={selectedRun.per_query} />
    {/if}
  </section>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(:root) {
    --bg: #0a0b10;
    --surface: #12131a;
    --accent: #4a9eff;
    --green: #66bb6a;
    --red: #ef5350;
    --yellow: #ffd54f;
  }

  :global(body) {
    background: var(--bg);
    color: #c9d1d9;
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      'Segoe UI',
      Roboto,
      sans-serif;
    line-height: 1.5;
  }

  .app {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1.5rem 1rem 3rem;
  }

  header {
    margin-bottom: 2rem;
  }

  h1 {
    font-size: 1.6rem;
    color: #e6edf3;
    font-weight: 700;
  }

  .accent {
    color: var(--accent);
  }

  .subtitle {
    color: #8b949e;
    font-size: 0.85rem;
    margin-top: 0.25rem;
  }

  .section {
    margin-bottom: 2.5rem;
    background: var(--surface);
    border: 1px solid #21262d;
    border-radius: 10px;
    padding: 1.5rem;
  }

  h2 {
    color: #e6edf3;
    font-size: 1rem;
    margin-bottom: 1rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .section-header h2 {
    margin-bottom: 0;
  }

  .run-select {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #c9d1d9;
    padding: 0.35rem 0.65rem;
    font-size: 0.82rem;
    cursor: pointer;
    outline: none;
  }

  .run-select:focus {
    border-color: var(--accent);
  }
</style>
