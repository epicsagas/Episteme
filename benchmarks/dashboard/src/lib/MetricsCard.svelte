<script>
  import HelpButton from './HelpButton.svelte';
  import Sparkline from './Sparkline.svelte';

  /** @type {{ title: string; metrics: Record<string, number>; format?: (v: number) => string; help?: { en: { title: string; lines: string[] }; ko: { title: string; lines: string[] } }; series?: number[]; delta?: number | null }} */
  let { title, metrics, format, help, series, delta } = $props();

  function fmt(v) {
    if (format) return format(v);
    if (Number.isInteger(v)) return v.toString();
    return v.toFixed(3);
  }

  function fmtDelta(d) {
    const abs = format ? format(Math.abs(d)) : Math.abs(d).toFixed(3);
    return (d >= 0 ? '+' : '-') + abs;
  }
</script>

<div class="card">
  <div class="card-header">
    <h3>{title}</h3>
    <div class="header-right">
      {#if delta != null}
        <span class="delta" class:up={delta >= 0} class:down={delta < 0}>{fmtDelta(delta)}</span>
      {/if}
      {#if help}
        <HelpButton {help} />
      {/if}
    </div>
  </div>
  <dl>
    {#each Object.entries(metrics) as [key, value]}
      <dt>{key}</dt>
      <dd>{fmt(value)}</dd>
    {/each}
  </dl>
  {#if series && series.length >= 2}
    <Sparkline {series} />
  {/if}
</div>

<style>
  .card {
    background: #161b22;
    border: 1px solid #21262d;
    border-radius: 8px;
    padding: 1rem;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .delta {
    font-size: 0.7rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
  }
  .delta.up { color: var(--green); background: rgba(102, 187, 106, 0.12); }
  .delta.down { color: var(--red); background: rgba(239, 83, 80, 0.12); }

  h3 {
    color: var(--accent);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.3rem 1rem;
    margin: 0;
  }

  dt {
    color: #8b949e;
    font-size: 0.82rem;
  }

  dd {
    color: #c9d1d9;
    font-size: 0.82rem;
    font-weight: 600;
    text-align: right;
    font-variant-numeric: tabular-nums;
    margin: 0;
  }
</style>
