<script>
  /** @type {{ regression: { status: string; prev_composite?: number; delta?: number; regressions?: string[] } | null, deltas?: Record<string, number | null> }} */
  let { regression, deltas = {} } = $props();

  let status = $derived(regression?.status ?? 'no_previous');
  let label = $derived(
    status === 'pass' ? 'PASS' :
    status === 'fail' ? 'FAIL' :
    'NO BASELINE'
  );
  let color = $derived(
    status === 'pass' ? 'green' :
    status === 'fail' ? 'red' :
    'gray'
  );

  const METRIC_LABELS = {
    composite: 'Composite',
    recall: 'Recall',
    precision: 'Precision',
    specificity: 'Specificity',
    smell_recall: 'Smell Recall',
  };
</script>

<div class="banner" class:green={color === 'green'} class:red={color === 'red'} class:gray={color === 'gray'}>
  <span class="status">{label}</span>
  {#if regression?.delta != null}
    <span class="delta" class:positive={regression.delta >= 0} class:negative={regression.delta < 0}>
      {regression.delta >= 0 ? '+' : ''}{regression.delta.toFixed(4)}
    </span>
  {:else if status === 'no_previous'}
    <span class="note">First run — no baseline for comparison</span>
  {/if}

  <!-- Per-metric delta badges -->
  {#if Object.keys(deltas).length > 0}
    <div class="metric-deltas">
      {#each Object.entries(deltas) as [key, d]}
        {@const lbl = METRIC_LABELS[key] ?? key}
        {#if d != null}
          <span class="metric-badge" class:up={d >= 0} class:down={d < 0}>
            {lbl} {d >= 0 ? '▲' : '▼'}{Math.abs(d).toFixed(3)}
          </span>
        {/if}
      {/each}
    </div>
  {/if}

  {#if regression?.regressions?.length}
    <ul class="warnings">
      {#each regression.regressions as msg}
        <li>{msg}</li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    font-weight: 600;
    flex-wrap: wrap;
  }

  .green {
    background: #1a3a1a;
    border: 1px solid var(--green);
  }

  .red {
    background: #3d0c0c;
    border: 1px solid var(--red);
  }

  .gray {
    background: #161b22;
    border: 1px solid #30363d;
  }

  .status {
    font-size: 1rem;
    letter-spacing: 0.1em;
  }

  .green .status { color: var(--green); }
  .red .status { color: var(--red); }
  .gray .status { color: #8b949e; }

  .delta {
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
  }

  .positive { color: var(--green); }
  .negative { color: var(--red); }

  .note {
    color: #8b949e;
    font-size: 0.82rem;
    font-weight: 400;
  }

  .metric-deltas {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-left: 0.5rem;
  }

  .metric-badge {
    font-size: 0.72rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
  }

  .metric-badge.up {
    color: var(--green);
    background: rgba(102, 187, 106, 0.12);
  }

  .metric-badge.down {
    color: var(--red);
    background: rgba(239, 83, 80, 0.15);
  }

  .warnings {
    margin: 0;
    padding-left: 1.2rem;
    color: var(--red);
    font-size: 0.8rem;
    font-weight: 400;
    flex-basis: 100%;
  }
</style>
