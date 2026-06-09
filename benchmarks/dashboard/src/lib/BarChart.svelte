<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart, BarController, BarElement,
    LinearScale, CategoryScale, Tooltip,
  } from 'chart.js';

  Chart.register(BarController, BarElement, LinearScale, CategoryScale, Tooltip);

  /** @type {{ data: Record<string, number>; color?: string }} */
  let { data, color = '#4a9eff' } = $props();

  let canvas;
  let chart;

  $effect(() => {
    if (!canvas || !data) return;

    const sorted = Object.entries(data).sort(([, a], [, b]) => b - a);
    const labels = sorted.map(([k]) => k);
    const values = sorted.map(([, v]) => v);

    if (chart) {
      chart.data.labels = labels;
      chart.data.datasets[0].data = values;
      chart.update();
      return;
    }

    chart = new Chart(canvas, {
      type: 'bar',
      data: {
        labels,
        datasets: [{ data: values, backgroundColor: color, borderRadius: 4 }],
      },
      options: {
        indexAxis: 'y',
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false } },
        scales: {
          x: { ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, beginAtZero: true },
          y: { ticks: { color: '#c9d1d9', font: { size: 11 } }, grid: { display: false } },
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
  .chart-wrap { height: 250px; position: relative; }
</style>
