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

  /** @type {{ label: string; timestamp: Date; latencyMean: number; latencyP95: number }[]} */
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
        label: 'mean',
        data: runs.map((r) => r.latencyMean),
        borderColor: '#4a9eff',
        backgroundColor: '#4a9eff22',
        fill: false,
        tension: 0.3,
        pointRadius: 5,
        pointHoverRadius: 8,
      },
      {
        label: 'p95',
        data: runs.map((r) => r.latencyP95),
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
              label(item) {
                return `${item.dataset.label}: ${item.parsed.y.toFixed(1)} ms`;
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
            title: {
              display: true,
              text: 'Latency (ms)',
              color: '#8b949e',
            },
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
    height: 250px;
    position: relative;
  }
</style>
