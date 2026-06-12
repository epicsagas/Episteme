<script>
  import { onDestroy } from 'svelte';
  import { Chart } from './chart-base.js';
  import { METRIC_COLORS } from './colors.js';

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
      mkds('Composite', runs.map((r) => r.composite), METRIC_COLORS.composite, 3),
      mkds('Recall', runs.map((r) => r.recall), METRIC_COLORS.recall),
      mkds('Precision', runs.map((r) => r.precision), METRIC_COLORS.precision),
      mkds('Specificity', runs.map((r) => r.specificity), METRIC_COLORS.specificity),
      mkds('Smell Recall', runs.map((r) => r.smell_recall), METRIC_COLORS.smell_recall),
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
