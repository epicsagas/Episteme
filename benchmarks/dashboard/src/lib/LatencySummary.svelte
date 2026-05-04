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

  /** @type {{ summary: import('../types').Summary }} */
  let { summary } = $props();

  let canvas;
  let chart;

  $effect(() => {
    if (!canvas || !summary) return;

    const lat = summary.latency_ms;
    const values = [lat.min, lat.p50, lat.mean, lat.p95, lat.p99, lat.max];
    const labels = ['min', 'p50', 'mean', 'p95', 'p99', 'max'];

    if (chart) {
      chart.data.datasets[0].data = values;
      chart.update();
      return;
    }

    chart = new Chart(canvas, {
      type: 'bar',
      data: {
        labels,
        datasets: [
          {
            label: 'Latency (ms)',
            data: values,
            backgroundColor: '#4a9eff99',
            borderColor: '#4a9eff',
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { labels: { color: '#c9d1d9' } },
          tooltip: {
            callbacks: {
              label: (item) => ` ${item.raw.toFixed(2)} ms`,
            },
          },
        },
        scales: {
          x: {
            ticks: { color: '#8b949e' },
            grid: { color: '#21262d' },
          },
          y: {
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

<div class="two-col">
  <div class="card summary-card">
    <h3>Quality</h3>
    <dl>
      <dt>hit@1</dt>
      <dd>{(summary.quality['hit@1'] * 100).toFixed(1)}%</dd>
      <dt>hit@3</dt>
      <dd>{(summary.quality['hit@3'] * 100).toFixed(1)}%</dd>
      <dt>hit@5</dt>
      <dd>{(summary.quality['hit@5'] * 100).toFixed(1)}%</dd>
      <dt>MRR@5</dt>
      <dd>{summary.quality['mrr@5'].toFixed(4)}</dd>
      <dt>NDCG@5</dt>
      <dd>{summary.quality['ndcg@5'].toFixed(4)}</dd>
    </dl>
    <h3 class="latency-title">Latency</h3>
    <dl>
      <dt>mean</dt>
      <dd>{summary.latency_ms.mean.toFixed(1)} ms</dd>
      <dt>p50</dt>
      <dd>{summary.latency_ms.p50.toFixed(1)} ms</dd>
      <dt>p95</dt>
      <dd>{summary.latency_ms.p95.toFixed(1)} ms</dd>
    </dl>
  </div>

  <div class="card chart-card">
    <h3>Latency Distribution</h3>
    <div class="chart-wrap">
      <canvas bind:this={canvas}></canvas>
    </div>
  </div>
</div>

<style>
  .two-col {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 1rem;
  }

  .card {
    background: var(--surface);
    border: 1px solid #21262d;
    border-radius: 8px;
    padding: 1.25rem;
  }

  h3 {
    color: var(--accent);
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .latency-title {
    margin-top: 1rem;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 1rem;
    margin: 0;
  }

  dt {
    color: #8b949e;
    font-size: 0.85rem;
  }

  dd {
    color: #c9d1d9;
    font-size: 0.85rem;
    font-weight: 600;
    margin: 0;
  }

  .chart-wrap {
    height: 220px;
    position: relative;
  }
</style>
