<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart,
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Tooltip,
    Legend,
  } from 'chart.js';

  Chart.register(
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Tooltip,
    Legend,
  );

  /** @type {{ label: string; timestamp: Date; hit1: number; mrr: number; ndcg: number; queries: number; latencyMean: number; topK: number }[]} */
  let { runs } = $props();

  let canvas;
  let chart;

  $effect(() => {
    if (!canvas || runs.length === 0) return;

    const labels = runs.map((r) =>
      r.timestamp.toLocaleString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      }),
    );

    const datasets = [
      {
        label: 'hit@1',
        data: runs.map((r) => r.hit1),
        borderColor: '#4a9eff',
        backgroundColor: '#4a9eff22',
        fill: false,
        tension: 0.3,
        pointRadius: 5,
        pointHoverRadius: 8,
      },
      {
        label: 'MRR@5',
        data: runs.map((r) => r.mrr),
        borderColor: '#66BB6A',
        backgroundColor: '#66BB6A22',
        fill: false,
        tension: 0.3,
        pointRadius: 5,
        pointHoverRadius: 8,
      },
      {
        label: 'NDCG@5',
        data: runs.map((r) => r.ndcg),
        borderColor: '#EF5350',
        backgroundColor: '#EF535022',
        fill: false,
        tension: 0.3,
        pointRadius: 5,
        pointHoverRadius: 8,
      },
    ];

    if (chart) {
      chart.data.labels = labels;
      chart.data.datasets = datasets;
      chart.update();
      return;
    }

    chart = new Chart(canvas, {
      type: 'line',
      data: { labels, datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
          mode: 'index',
          intersect: false,
        },
        plugins: {
          legend: {
            labels: { color: '#c9d1d9' },
          },
          tooltip: {
            callbacks: {
              afterBody(items) {
                const idx = items[0]?.dataIndex;
                if (idx == null) return [];
                const r = runs[idx];
                return [
                  `queries: ${r.queries}`,
                  `latency mean: ${r.latencyMean.toFixed(1)} ms`,
                  `top_k: ${r.topK}`,
                ];
              },
            },
          },
        },
        scales: {
          x: {
            ticks: { color: '#8b949e', maxRotation: 45 },
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

<div class="chart-wrap">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .chart-wrap {
    height: 300px;
    position: relative;
  }
</style>
