<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart, LineController, LineElement, PointElement,
    LinearScale, CategoryScale, Tooltip, Legend,
  } from 'chart.js';

  Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Tooltip, Legend);

  /** @type {{ runs: Array<{ label: string; timestamp: Date; composite: number; recall: number; precision: number; specificity: number; smell_recall: number }> }} */
  let { runs } = $props();

  let canvas;
  let chart;

  $effect(() => {
    if (!canvas || runs.length === 0) return;

    const labels = runs.map((r) =>
      r.timestamp.toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
    );

    const mkds = (label, data, color, width = 2) => ({
      label, data,
      borderColor: color,
      backgroundColor: color + '22',
      fill: false,
      tension: 0.3,
      pointRadius: 5,
      pointHoverRadius: 8,
      borderWidth: width,
    });

    const datasets = [
      mkds('Composite', runs.map((r) => r.composite), '#e6edf3', 3),
      mkds('Recall', runs.map((r) => r.recall), '#4a9eff'),
      mkds('Precision', runs.map((r) => r.precision), '#66bb6a'),
      mkds('Specificity', runs.map((r) => r.specificity), '#ffd54f'),
      mkds('Smell Recall', runs.map((r) => r.smell_recall), '#ef5350'),
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
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { labels: { color: '#c9d1d9' } },
        },
        scales: {
          x: { ticks: { color: '#8b949e', maxRotation: 45 }, grid: { color: '#21262d' } },
          y: { min: 0, max: 1.05, ticks: { color: '#8b949e' }, grid: { color: '#21262d' } },
        },
      },
    });
  });

  onDestroy(() => { chart?.destroy(); });
</script>

<div class="chart-wrap">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .chart-wrap { height: 300px; position: relative; }
</style>
