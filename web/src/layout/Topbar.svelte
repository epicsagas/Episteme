<script lang="ts">
  import { isDark, toggle } from '../stores/theme.svelte.ts';
  import { getStatus, getBaseUrl } from '../stores/connection.svelte.ts';
  import { search } from '../api/endpoints.ts';
  import { navigate } from '../router/index.svelte.ts';
  import type { SearchResult } from '../api/types.ts';

  let status = $derived(getStatus());

  let searchQuery = $state('');
  let searchResults: SearchResult[] = $state([]);
  let showDropdown = $state(false);
  let activeIndex = $state(-1);
  let searchTimer: ReturnType<typeof setTimeout>;

  function handleSearchInput() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(async () => {
      if (searchQuery.length < 2) {
        searchResults = [];
        showDropdown = false;
        activeIndex = -1;
        return;
      }
      try {
        const response = await search(getBaseUrl(), searchQuery, 8);
        searchResults = response.results;
        showDropdown = searchResults.length > 0;
        activeIndex = -1;
      } catch {
        searchResults = [];
        showDropdown = false;
        activeIndex = -1;
      }
    }, 300);
  }

  function selectResult(id: string) {
    showDropdown = false;
    searchQuery = '';
    searchResults = [];
    activeIndex = -1;
    navigate({ page: 'entity', id, from: 'explorer' });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown || searchResults.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, searchResults.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter' && activeIndex >= 0) {
      e.preventDefault();
      selectResult(searchResults[activeIndex].entity_id);
    } else if (e.key === 'Escape') {
      showDropdown = false;
      activeIndex = -1;
    }
  }

  function closeDropdownOnFocusout(event: FocusEvent) {
    if (event.relatedTarget && (event.relatedTarget as HTMLElement).closest('[data-search-dropdown]')) {
      return;
    }
    showDropdown = false;
  }

  function entityTypeIcon(type: string): string {
    const map: Record<string, string> = {
      pattern: 'design_services',
      refactoring: 'transform',
      law: 'gavel',
      smell: 'warning',
      insight: 'lightbulb',
    };
    return map[type] || 'circle';
  }
</script>

<header class="flex justify-between items-center px-4 h-[var(--topbar-height)] border-b shrink-0
  bg-[var(--color-surface)]/80 backdrop-blur-md border-[var(--color-outline-variant)]">
  <div class="flex items-center gap-4">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="relative" role="combobox" aria-expanded={showDropdown} onkeydown={handleKeydown}>
      <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2
        text-[var(--color-on-surface-variant)] pointer-events-none text-[18px]">search</span>
      <input
        id="global-search"
        type="text"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        onblur={closeDropdownOnFocusout}
        onfocus={() => { if (searchResults.length > 0) showDropdown = true; }}
        placeholder="Search entities... (⌘K)"
        class="bg-[var(--color-surface-container-low)] border border-[var(--color-outline-variant)]
          rounded-lg pl-9 pr-10 py-1.5 text-sm w-72 outline-none
          focus:ring-1 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
          text-[var(--color-on-surface)] placeholder:text-[var(--color-outline)]
          transition-colors"
      />
      <kbd class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-mono
        text-[var(--color-outline)] border border-[var(--color-outline-variant)] px-1.5 py-0.5 rounded
        bg-[var(--color-surface-container)]">⌘K</kbd>

      {#if showDropdown && searchResults.length > 0}
        <div data-search-dropdown class="absolute top-full left-0 mt-2 w-96 glass-panel shadow-xl z-50
          max-h-80 overflow-y-auto animate-fade-in">
          {#each searchResults as result, i}
            <button
              class="w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors
                {i === activeIndex
                  ? 'bg-[var(--color-primary)]/10'
                  : 'hover:bg-[var(--color-surface-container-high)]/50'}
                border-b border-[var(--color-outline-variant)]/20 last:border-0"
              onclick={() => selectResult(result.entity_id)}
              onmouseenter={() => activeIndex = i}
            >
              <div class="w-7 h-7 rounded-md flex items-center justify-center shrink-0"
                style="background: color-mix(in srgb, var(--color-{result.type}) 15%, transparent);
                       border: 1px solid color-mix(in srgb, var(--color-{result.type}) 25%, transparent);">
                <span class="material-symbols-outlined text-[14px]"
                  style="color: var(--color-{result.type})">{entityTypeIcon(result.type)}</span>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-sm text-[var(--color-on-surface)] truncate">{result.title}</p>
                <p class="text-[10px] text-[var(--color-on-surface-variant)] font-mono">{result.entity_id}</p>
              </div>
              <span class="text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded
                text-[var(--color-on-surface-variant)] bg-[var(--color-surface-container-high)]/50">{result.type}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="flex items-center gap-3">
    <div class="flex items-center gap-1 border-l border-[var(--color-outline-variant)] pl-3">
      <div class="w-1.5 h-1.5 rounded-full
        {status === 'connected' ? 'bg-[var(--color-rel-solves)]' : status === 'connecting' ? 'bg-[var(--color-law)]' : 'bg-[var(--color-error)]'}">
      </div>
      <span class="text-[10px] text-[var(--color-on-surface-variant)]">
        {status === 'connected' ? 'Connected' : status === 'connecting' ? 'Connecting...' : 'Offline'}
      </span>
    </div>

    <button
      onclick={toggle}
      class="p-1.5 rounded-lg text-[var(--color-on-surface-variant)]
        hover:text-[var(--color-primary)] hover:bg-[var(--color-surface-container-high)]/50 transition-colors"
    >
      <span class="material-symbols-outlined text-[18px]">
        {isDark() ? 'light_mode' : 'dark_mode'}
      </span>
    </button>
  </div>
</header>
