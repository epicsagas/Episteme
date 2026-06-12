<script>
  import CompositeTrend from './lib/CompositeTrend.svelte';
  import RegressionBanner from './lib/RegressionBanner.svelte';
  import MetricsCard from './lib/MetricsCard.svelte';
  import BarChart from './lib/BarChart.svelte';
  import QueryTable from './lib/QueryTable.svelte';
  import ViolationTable from './lib/ViolationTable.svelte';
  import HelpButton from './lib/HelpButton.svelte';
  import { i18n, toggleLocale } from './lib/i18n.svelte';
  import { help } from './lib/help.js';
  import { loadEvalRuns, toTrendRuns, toSeries, computeDeltas } from './lib/data.js';
  import { BAR_CHART_RED, BAR_CHART_GREEN } from './lib/colors.js';

  const rawFiles = import.meta.glob('/results/*.json', { eager: true });

  const runs = loadEvalRuns(rawFiles);
  const trendRuns = toTrendRuns(runs);
  const series = toSeries(trendRuns);

  let selectedIndex = $state(runs.length - 1);
  let selectedRun = $derived(runs[selectedIndex]);

  function pct(v) { return (v * 100).toFixed(1) + '%'; }

  let prevRun = $derived(selectedIndex > 0 ? runs[selectedIndex - 1] : null);

  let deltas = $derived(computeDeltas(selectedRun?.composite_score, prevRun?.composite_score));
</script>

<div class="app">
  <header>
    <div class="header-row">
      <h1>Episteme <span class="accent">Eval</span> Dashboard</h1>
      <button class="lang-toggle" onclick={toggleLocale}>
        {i18n.locale === 'en' ? '한국어' : 'EN'}
      </button>
    </div>
    <p class="subtitle">{runs.length} eval runs | {selectedRun?.git_commit}</p>
  </header>

  <!-- Composite Trend -->
  <section class="section">
    <div class="section-header">
      <h2>Composite Score Trend</h2>
      <HelpButton help={help.compositeTrend} />
    </div>
    <CompositeTrend runs={trendRuns} />
  </section>

  <!-- Run Details -->
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
      <RegressionBanner regression={selectedRun.regression} {deltas} />
      <div class="composite-cards">
        <MetricsCard title="Composite" metrics={{ score: selectedRun.composite_score.composite }} format={pct} help={help.composite} series={series.composite} delta={deltas.composite} />
        <MetricsCard title="Recall" metrics={{ recall: selectedRun.composite_score.recall }} format={pct} help={help.recall} series={series.recall} delta={deltas.recall} />
        <MetricsCard title="Precision" metrics={{ precision: selectedRun.composite_score.precision }} format={pct} help={help.precision} series={series.precision} delta={deltas.precision} />
        <MetricsCard title="Specificity" metrics={{ specificity: selectedRun.composite_score.specificity }} format={pct} help={help.specificity} series={series.specificity} delta={deltas.specificity} />
        <MetricsCard title="Smell Recall" metrics={{ recall: selectedRun.composite_score.smell_recall }} format={pct} help={help.smellRecall} series={series.smell_recall} delta={deltas.smell_recall} />
      </div>
    {/if}
  </section>

  <!-- Search Positive -->
  <section class="section">
    <div class="section-header">
      <h2>Search Positive</h2>
      <HelpButton help={help.searchPositive} />
    </div>
    {#if selectedRun?.suites?.search_positive}
      <MetricsCard title="Metrics" metrics={selectedRun.suites.search_positive.metrics} format={pct} help={help.searchPositiveMetrics} />
      <QueryTable perQuery={selectedRun.suites.search_positive.per_query} />
    {:else}
      <p class="empty">No data</p>
    {/if}
  </section>

  <!-- Search Negative -->
  <section class="section">
    <div class="section-header">
      <h2>Search Negative</h2>
      <HelpButton help={help.searchNegative} />
    </div>
    {#if selectedRun?.suites?.search_negative}
      {@const sn = selectedRun.suites.search_negative}
      <MetricsCard title="FP Rates" metrics={{ 'fp@1': sn.metrics['fp@1'], 'fp@3': sn.metrics['fp@3'], 'fp@5': sn.metrics['fp@5'] }} format={pct} help={help.searchNegativeFp} />
      <MetricsCard title="Specificity" metrics={{ specificity: sn.metrics.specificity, true_negatives: sn.metrics.true_negatives, total: sn.metrics.total }} help={help.searchNegativeSpecificity} />
      <ViolationTable perQuery={sn.per_query} />
    {:else}
      <p class="empty">No data</p>
    {/if}
  </section>

  <!-- Smell Negative -->
  <section class="section">
    <div class="section-header">
      <h2>Smell Negative (False Positive Rate)</h2>
      <HelpButton help={help.smellNegative} />
    </div>
    {#if selectedRun?.suites?.smell_negative}
      {@const sm = selectedRun.suites.smell_negative}
      <MetricsCard title="FP Rate" metrics={{ fp_rate: sm.metrics.fp_rate, fp_count: sm.metrics.fp_count, total: sm.metrics.total, specificity: sm.metrics.specificity }} help={help.smellNegativeFp} />
      <h3>Per Detector</h3>
      <BarChart data={sm.metrics.per_detector} color={BAR_CHART_RED} />
      <h3>Per Language</h3>
      <table class="lang-table">
        <thead>
          <tr><th>Language</th><th>FP Rate</th></tr>
        </thead>
        <tbody>
          {#each Object.entries(sm.metrics.per_language).sort(([,a],[,b]) => b - a) as [lang, rate]}
            <tr>
              <td>{lang}</td>
              <td class="rate" style="color: {rate === 0 ? 'var(--green)' : rate < 0.5 ? 'var(--yellow)' : 'var(--red)'}">{(rate * 100).toFixed(0)}%</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="empty">No data</p>
    {/if}
  </section>

  <!-- Analyze Positive -->
  <section class="section">
    <div class="section-header">
      <h2>Analyze Positive (Smell Detection Recall)</h2>
      <HelpButton help={help.analyzePositive} />
    </div>
    {#if selectedRun?.suites?.analyze_positive}
      {@const ap = selectedRun.suites.analyze_positive}
      <MetricsCard title="Recall" metrics={{ recall: ap.metrics.recall, hits: ap.metrics.hits, total: ap.metrics.total }} help={help.analyzePositiveRecall} />
      <h3>Per Smell Recall</h3>
      <BarChart data={ap.metrics.per_smell_recall} color={BAR_CHART_GREEN} format={pct} />
    {:else}
      <p class="empty">No data</p>
    {/if}
  </section>

  <!-- Traversal -->
  <section class="section">
    <div class="section-header">
      <h2>Traversal</h2>
      <HelpButton help={help.traversal} />
    </div>
    {#if selectedRun?.suites?.traversal}
      {@const tr = selectedRun.suites.traversal}
      <div class="traversal-grid">
        <MetricsCard title="Neighbors" metrics={{ recall: tr.metrics.neighbors.recall, hits: tr.metrics.neighbors.hits, total: tr.metrics.neighbors.total }} help={help.traversalNeighbors} />
        <MetricsCard title="Paths" metrics={{ recall: tr.metrics.paths.recall, hits: tr.metrics.paths.hits, total: tr.metrics.paths.total }} help={help.traversalPaths} />
      </div>
    {:else}
      <p class="empty">No data</p>
    {/if}
  </section>
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }

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
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.5;
  }

  .app { max-width: 1200px; margin: 0 auto; padding: 1.5rem 1rem 3rem; }

  header { margin-bottom: 2rem; }

  .header-row { display: flex; align-items: center; justify-content: space-between; }

  h1 { font-size: 1.6rem; color: #e6edf3; font-weight: 700; }
  .accent { color: var(--accent); }
  .subtitle { color: #8b949e; font-size: 0.85rem; margin-top: 0.25rem; }

  .lang-toggle {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #8b949e;
    padding: 0.3rem 0.65rem;
    font-size: 0.78rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }
  .lang-toggle:hover { border-color: var(--accent); color: var(--accent); }

  .section {
    margin-bottom: 2.5rem;
    background: var(--surface);
    border: 1px solid #21262d;
    border-radius: 10px;
    padding: 1.5rem;
  }

  h2 { color: #e6edf3; font-size: 1rem; margin-bottom: 1rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; }
  h3 { color: var(--accent); font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.08em; margin: 1.25rem 0 0.75rem; }

  .section-header { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; }
  .section-header h2 { margin-bottom: 0; }

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
  .run-select:focus { border-color: var(--accent); }

  .composite-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 0.75rem;
  }

  .traversal-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .lang-table {
    width: 100%;
    max-width: 300px;
    border-collapse: collapse;
    font-size: 0.83rem;
    margin-top: 0.5rem;
  }
  .lang-table th { background: #161b22; color: #8b949e; text-transform: uppercase; font-size: 0.75rem; padding: 0.5rem 0.75rem; text-align: left; }
  .lang-table td { padding: 0.45rem 0.75rem; border-top: 1px solid #21262d; }
  .lang-table .rate { text-align: right; font-weight: 600; font-variant-numeric: tabular-nums; }

  .empty { color: #8b949e; font-size: 0.85rem; }
</style>
