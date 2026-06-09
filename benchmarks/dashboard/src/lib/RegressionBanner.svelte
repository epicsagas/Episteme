<script>
  /** @type {{ regression: { status: string; prev_composite?: number; delta?: number; regressions?: string[] } | null }} */
  let { regression } = $props();

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

  .warnings {
    margin: 0;
    padding-left: 1.2rem;
    color: var(--red);
    font-size: 0.8rem;
    font-weight: 400;
  }
</style>
