<script lang="ts">
  import { createInsight } from '../api/endpoints.ts';
  import { getBaseUrl } from '../stores/connection.svelte.ts';
  import type { CreateInsightResponse } from '../api/types.ts';

  let text = $state('');
  let tagsInput = $state('');
  let submitting = $state(false);
  let result: CreateInsightResponse | null = $state(null);
  let errorMsg: string | null = $state(null);

  let { oncreated }: { oncreated?: (id: string) => void } = $props();

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!text.trim() || submitting) return;

    submitting = true;
    errorMsg = null;
    result = null;

    try {
      const tags = tagsInput.trim()
        ? tagsInput.split(',').map(t => t.trim()).filter(t => t.length > 0)
        : undefined;

      const res = await createInsight(getBaseUrl(), text.trim(), tags);
      if (res.error) {
        errorMsg = res.error;
      } else {
        result = res;
        text = '';
        tagsInput = '';
        oncreated?.(res.id);
      }
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Failed to create insight';
    } finally {
      submitting = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="space-y-3">
  <textarea
    bind:value={text}
    placeholder="Share an insight... e.g., 'We prefer composition over inheritance for payment strategies'"
    rows="3"
    class="w-full bg-[var(--color-surface-container-lowest)] border border-[var(--color-outline-variant)]
      rounded-lg px-3 py-2 text-sm text-[var(--color-on-surface)] placeholder:text-[var(--color-on-surface-variant)]/50
      focus:outline-none focus:border-[var(--color-primary)] resize-none"
  ></textarea>

  <input
    type="text"
    bind:value={tagsInput}
    placeholder="Tags (comma-separated, optional)"
    class="w-full bg-[var(--color-surface-container-lowest)] border border-[var(--color-outline-variant)]
      rounded-lg px-3 py-2 text-sm text-[var(--color-on-surface)] placeholder:text-[var(--color-on-surface-variant)]/50
      focus:outline-none focus:border-[var(--color-primary)]"
  />

  {#if errorMsg}
    <p class="text-xs text-[var(--color-error)]">{errorMsg}</p>
  {/if}

  <button
    type="submit"
    disabled={!text.trim() || submitting}
    class="w-full py-2 rounded-lg text-sm font-bold transition-colors
      bg-[var(--color-primary)] text-[var(--color-on-primary)]
      disabled:opacity-40 disabled:cursor-not-allowed
      hover:bg-[var(--color-primary-container)] hover:text-[var(--color-on-primary-container)]"
  >
    {submitting ? 'Creating...' : 'Add Insight'}
  </button>

  {#if result}
    <div class="mt-3 p-3 rounded-lg bg-[var(--color-surface-container-low)] border border-[var(--color-outline-variant)]/30">
      <p class="text-xs font-bold text-[var(--color-on-surface)] mb-1">
        Created: <span class="font-mono text-[var(--color-insight)]">{result.id}</span>
      </p>
      {#if result.auto_links.length > 0}
        <p class="text-[10px] uppercase tracking-wider text-[var(--color-on-surface-variant)] mt-2 mb-1">Auto-detected links</p>
        <div class="flex flex-wrap gap-1">
          {#each result.auto_links as link}
            <span class="text-[10px] px-1.5 py-0.5 rounded
              bg-[var(--color-surface-container-high)] text-[var(--color-on-surface-variant)]">
              {link.entity_id}
              <span class="opacity-60">({link.link_type})</span>
            </span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</form>
