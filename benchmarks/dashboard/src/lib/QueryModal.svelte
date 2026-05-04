<script>
  /** @type {{ query: import('../types').PerQuery | null; onclose: () => void }} */
  let { query, onclose } = $props();

  function handleKeydown(e) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if query}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={onclose}>
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <button class="close" onclick={onclose} aria-label="Close">✕</button>
      <h2 class="modal-title">{query.query}</h2>

      <div class="section">
        <h3>Relevant IDs</h3>
        <div class="chips">
          {#each query.relevant_ids as id}
            <span class="chip relevant">{id}</span>
          {/each}
        </div>
      </div>

      <div class="section">
        <h3>Top IDs returned</h3>
        <div class="chips">
          {#each query.top_ids as id}
            <span class="chip" class:match={query.relevant_ids.includes(id)}>{id}</span>
          {/each}
        </div>
      </div>

      <div class="section metrics">
        <div class="metric">
          <span class="label">hit@1</span>
          <span class="value" class:good={query['hit@1'] === 1} class:bad={query['hit@1'] === 0}>
            {query['hit@1'] === 1 ? '✓' : '✗'}
          </span>
        </div>
        <div class="metric">
          <span class="label">RR@5</span>
          <span class="value">{query['rr@5'].toFixed(4)}</span>
        </div>
        <div class="metric">
          <span class="label">NDCG@5</span>
          <span class="value">{query['ndcg@5'].toFixed(4)}</span>
        </div>
        <div class="metric">
          <span class="label">Latency mean</span>
          <span class="value">{query.latency_mean_ms.toFixed(1)} ms</span>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: #00000099;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    background: var(--surface);
    border: 1px solid #30363d;
    border-radius: 10px;
    padding: 1.75rem;
    max-width: 520px;
    width: 90%;
    position: relative;
  }

  .close {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    background: none;
    border: none;
    color: #8b949e;
    cursor: pointer;
    font-size: 1rem;
  }

  .close:hover {
    color: #c9d1d9;
  }

  .modal-title {
    color: #c9d1d9;
    margin: 0 0 1.25rem;
    font-size: 1rem;
  }

  .section {
    margin-bottom: 1rem;
  }

  h3 {
    color: var(--accent);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0 0 0.5rem;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .chip {
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 4px;
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
    color: #8b949e;
  }

  .chip.relevant {
    border-color: var(--accent);
    color: var(--accent);
  }

  .chip.match {
    border-color: var(--green);
    color: var(--green);
  }

  .metrics {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .metric {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .label {
    color: #8b949e;
    font-size: 0.75rem;
  }

  .value {
    color: #c9d1d9;
    font-size: 1rem;
    font-weight: 600;
  }

  .value.good {
    color: var(--green);
  }

  .value.bad {
    color: var(--red);
  }
</style>
