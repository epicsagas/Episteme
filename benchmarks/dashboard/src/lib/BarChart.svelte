<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart, BarController, BarElement,
    LinearScale, CategoryScale, Tooltip,
  } from 'chart.js';

  Chart.register(BarController, BarElement, LinearScale, CategoryScale, Tooltip);

  /** @type {{ data: Record<string, number>; color?: string; format?: (v: number) => string }} */
  let { data, color = '#4a9eff', format } = $props();

  let canvas;
  let chart;

  function fmt(v) {
    if (format) return format(v);
    if (Number.isInteger(v)) return v.toString();
    return v.toFixed(2);
  }

  // Per-chart plugin: draws each bar's value just past the bar end (horizontal bars).
  const valueLabels = {
    id: 'bar-value-labels',
    afterDatasetsDraw(chart) {
      const { ctx } = chart;
      const meta = chart.getDatasetMeta(0);
      if (!meta || !meta.data) return;
      ctx.save();
      ctx.font = '600 11px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
      ctx.fillStyle = '#c9d1d9';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'middle';
      meta.data.forEach((bar, i) => {
        const value = chart.data.datasets[0].data[i];
        if (value == null) return;
        ctx.fillText(fmt(value), bar.x + 6, bar.y);
      });
      ctx.restore();
    },
  };

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
        layout: { padding: { right: 36 } },
        plugins: { legend: { display: false } },
        scales: {
          x: { ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, beginAtZero: true },
          y: { ticks: { color: '#c9d1d9', font: { size: 11 } }, grid: { display: false } },
        },
      },
      plugins: [valueLabels],
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
