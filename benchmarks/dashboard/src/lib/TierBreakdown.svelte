<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart,
    BarController,
    BarElement,
    LinearScale,
    CategoryScale,
    Tooltip,
    Legend,
  } from 'chart.js';

  Chart.register(BarController, BarElement, LinearScale, CategoryScale, Tooltip, Legend);

  /** @type {{ per_tier?: Record<string, { 'hit@1': number; 'hit@3': number; 'hit@5': number; 'mrr@5': number; 'ndcg@5': number; queries: number }>; per_category?: Record<string, { 'hit@1': number; 'hit@3': number; 'hit@5': number; 'mrr@5': number; 'ndcg@5': number; queries: number }> }} */
  let { summary } = $props();

  let canvas;
  let chart;

  const tierColors = {
    easy: '#66BB6A',
    medium: '#FFD54F',
    hard: '#EF5350',
  };

  const categoryColors = {
    exact_name: '#4a9eff',
    partial_match: '#AB47BC',
    conceptual: '#FF7043',
    cross_domain: '#26C6DA',
  };

  $effect(() => {
    if (!canvas || !summary?.per_tier) return;

    const tiers = Object.keys(summary.per_tier);
    const labels = tiers.map((t) => `${t} (${summary.per_tier[t].queries})`);

    const metrics = ['hit@1', 'hit@3', 'hit@5', 'mrr@5', 'ndcg@5'];
    const metricColors = ['#4a9eff', '#66BB6A', '#FFD54F', '#AB47BC', '#FF7043'];

    const datasets = metrics.map((metric, i) => ({
      label: metric,
      data: tiers.map((t) => summary.per_tier[t][metric] ?? 0),
      backgroundColor: metricColors[i] + '99',
      borderColor: metricColors[i],
      borderWidth: 1,
    }));

    if (chart) {
      chart.data.labels = labels;
      chart.data.datasets = datasets;
      chart.update();
      return;
    }

    chart = new Chart(canvas, {
      type: 'bar',
      data: { labels, datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { labels: { color: '#c9d1d9' } },
          tooltip: {
            callbacks: {
              label: (item) => ` ${item.dataset.label}: ${(item.raw * 100).toFixed(1)}%`,
            },
          },
        },
        scales: {
          x: {
            ticks: { color: '#8b949e' },
            grid: { color: '#21262d' },
          },
          y: {
            min: 0,
            max: 1.05,
            ticks: { color: '#8b949e' },
            grid: { color: '#21262d' },
          },
        },
      },
    });
  });

  onDestroy(() => {
    chart?.destroy();
  });
</script>

{#if summary?.per_tier}
  <div class="breakdown">
    <div class="chart-wrap">
      <canvas bind:this={canvas}></canvas>
    </div>
    {#if summary.per_category}
      <div class="category-table">
        <h4>By Category</h4>
        <table>
          <thead>
            <tr>
              <th>Category</th>
              <th>hit@1</th>
              <th>hit@5</th>
              <th>NDCG</th>
              <th>#</th>
            </tr>
          </thead>
          <tbody>
            {#each Object.entries(summary.per_category) as [cat, data]}
              <tr>
                <td class="cat-label">{cat.replace(/_/g, ' ')}</td>
                <td class="num">{(data['hit@1'] * 100).toFixed(0)}%</td>
                <td class="num">{(data['hit@5'] * 100).toFixed(0)}%</td>
                <td class="num">{data['ndcg@5'].toFixed(3)}</td>
                <td class="num">{data.queries}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
{:else}
  <p class="no-data">No tier data available for this run (older format).</p>
{/if}

<style>
  .breakdown {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    align-items: start;
  }

  .chart-wrap {
    height: 280px;
    position: relative;
  }

  .category-table {
    overflow-x: auto;
  }

  h4 {
    color: var(--accent);
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0 0 0.5rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  thead th {
    background: #161b22;
    color: #8b949e;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.72rem;
    padding: 0.4rem 0.6rem;
    text-align: left;
  }

  tbody td {
    padding: 0.4rem 0.6rem;
    color: #c9d1d9;
    border-top: 1px solid #21262d;
  }

  .cat-label {
    color: #8b949e;
  }

  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .no-data {
    color: #8b949e;
    font-size: 0.85rem;
    font-style: italic;
  }
</style>
