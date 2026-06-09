<script>
  import HelpButton from './HelpButton.svelte';

  /** @type {{ title: string; metrics: Record<string, number>; format?: (v: number) => string; help?: { en: { title: string; lines: string[] }; ko: { title: string; lines: string[] } } }} */
  let { title, metrics, format, help } = $props();

  function fmt(v) {
    if (format) return format(v);
    if (Number.isInteger(v)) return v.toString();
    return v.toFixed(3);
  }
</script>

<div class="card">
  <div class="card-header">
    <h3>{title}</h3>
    {#if help}
      <HelpButton {help} />
    {/if}
  </div>
  <dl>
    {#each Object.entries(metrics) as [key, value]}
      <dt>{key}</dt>
      <dd>{fmt(value)}</dd>
    {/each}
  </dl>
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
