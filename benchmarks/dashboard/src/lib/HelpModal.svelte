<script>
  /** @type {{ title: string; lines: string[]; open: boolean; onclose: () => void }} */
  let { title, lines, open, onclose } = $props();

  function handleKeydown(e) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={onclose}>
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <button class="close" onclick={onclose} aria-label="Close">✕</button>
      <h2>{title}</h2>
      <ul>
        {#each lines as line}
          <li>{@html line}</li>
        {/each}
      </ul>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed; inset: 0;
    background: #00000099;
    display: flex; align-items: center; justify-content: center;
    z-index: 200;
  }

  .modal {
    background: var(--surface);
    border: 1px solid #30363d;
    border-radius: 10px;
    padding: 1.5rem 1.75rem;
    max-width: 560px; width: 90%;
    position: relative;
  }

  .close {
    position: absolute; top: 0.75rem; right: 0.75rem;
    background: none; border: none; color: #8b949e;
    cursor: pointer; font-size: 1rem;
  }
  .close:hover { color: #c9d1d9; }

  h2 { color: var(--accent); font-size: 0.9rem; margin: 0 0 1rem; text-transform: uppercase; letter-spacing: 0.06em; }

  ul { list-style: none; padding: 0; margin: 0; }
  li {
    color: #c9d1d9; font-size: 0.85rem; line-height: 1.6;
    padding: 0.35rem 0; border-bottom: 1px solid #21262d;
  }
  li:last-child { border-bottom: none; }

  li :global(strong) { color: #e6edf3; }
  li :global(code) {
    background: #21262d; padding: 0.1rem 0.35rem;
    border-radius: 3px; font-size: 0.82rem; color: var(--accent);
  }
</style>
