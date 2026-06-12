<script>
  import { onDestroy } from 'svelte';
  import { Chart } from './chart-base.js';
  import { METRIC_COLORS } from './colors.js';

  /** @type {{ series: number[]; color?: string }} */
  let { series, color = METRIC_COLORS.recall } = $props();

  let canvas;
  let chart;

  $effect(() => {
    if (!canvas || !series || series.length === 0) return;

    const data = series.map((v) => (v == null ? 0 : v));
    const labels = data.map((_, i) => i);

    if (chart) {
      chart.data.labels = labels;
      chart.data.datasets[0].data = data;
      chart.update();
      return;
    }

    chart = new Chart(canvas, {
      type: 'line',
      data: {
        labels,
        datasets: [{
          data,
          borderColor: color,
          backgroundColor: color + '22',
          borderWidth: 1.5,
          pointRadius: 0,
          tension: 0.3,
          fill: true,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: { legend: { display: false }, tooltip: { enabled: false } },
        scales: { x: { display: false }, y: { display: false } },
      },
    });
  });

  onDestroy(() => { chart?.destroy(); });
</script>

<div class="spark"><canvas bind:this={canvas}></canvas></div>

<style>
  .spark { height: 32px; position: relative; margin-top: 0.6rem; }
</style>
