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
  let searchTimer: ReturnType<typeof setTimeout>;

  function handleSearchInput() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(async () => {
      if (searchQuery.length < 2) {
        searchResults = [];
        showDropdown = false;
        return;
      }
      try {
        const response = await search(getBaseUrl(), searchQuery, 5);
        searchResults = response.results;
        showDropdown = searchResults.length > 0;
      } catch {
        searchResults = [];
        showDropdown = false;
      }
    }, 300);
  }

  function selectResult(id: string) {
    showDropdown = false;
    searchQuery = '';
    searchResults = [];
    navigate({ page: 'entity', id });
  }

  function closeDropdownOnFocusout(event: FocusEvent) {
    if (event.relatedTarget && (event.relatedTarget as HTMLElement).closest('[data-search-dropdown]')) {
      return;
    }
    showDropdown = false;
  }
</script>

<header class="flex justify-between items-center px-6 h-16 border-b shrink-0
  bg-[var(--color-surface)]/80 backdrop-blur-md border-[var(--color-outline-variant)]">
  <div class="flex items-center gap-6">
    <span class="text-sm font-bold text-[var(--color-on-surface)]">Knowledge Graph</span>
    <div class="relative">
      <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2
        text-[var(--color-on-surface-variant)] pointer-events-none text-[20px]">search</span>
      <input
        id="global-search"
        type="text"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        onblur={closeDropdownOnFocusout}
        placeholder="Search entities... (⌘K)"
        class="bg-[var(--color-surface-container-low)] border border-[var(--color-outline-variant)]
          rounded-full pl-10 pr-10 py-1.5 text-sm w-80 outline-none
          focus:ring-1 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
          text-[var(--color-on-surface)] placeholder:text-[var(--color-outline)]"
      />
      <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-mono
        text-[var(--color-outline)] border border-[var(--color-outline-variant)] px-1 rounded">⌘K</span>
      {#if showDropdown && searchResults.length > 0}
        <div data-search-dropdown class="absolute top-full left-0 mt-2 w-80 bg-[var(--color-surface-container)] border border-[var(--color-outline-variant)] rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto">
          {#each searchResults as result}
            <button
              class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-[var(--color-surface-container-high)]/50 transition-colors border-b border-[var(--color-outline-variant)]/20 last:border-0"
              onclick={() => selectResult(result.entity_id)}
            >
              <span class="text-xs font-mono text-[var(--color-on-surface-variant)]">{result.entity_id}</span>
              <span class="text-sm text-[var(--color-on-surface)] flex-1 truncate">{result.title}</span>
              <span class="text-[10px] uppercase text-[var(--color-on-surface-variant)]">{result.type}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="flex items-center gap-4">
    <div class="flex items-center gap-1">
      <button disabled title="Coming soon" class="p-2 rounded-full text-[var(--color-on-surface-variant)] opacity-50 cursor-not-allowed">
        <span class="material-symbols-outlined text-[20px]">history</span>
      </button>
      <button disabled title="Coming soon" class="p-2 rounded-full text-[var(--color-on-surface-variant)] relative opacity-50 cursor-not-allowed">
        <span class="material-symbols-outlined text-[20px]">notifications</span>
      </button>
    </div>

    <div class="flex items-center gap-1 border-l border-[var(--color-outline-variant)] pl-4">
      <div class="w-2 h-2 rounded-full
        {status === 'connected' ? 'bg-[var(--color-rel-solves)]' : status === 'connecting' ? 'bg-[var(--color-law)]' : 'bg-[var(--color-error)]'}">
      </div>
      <span class="text-[10px] text-[var(--color-on-surface-variant)]">
        {status === 'connected' ? 'Connected' : status === 'connecting' ? 'Connecting...' : 'Offline'}
      </span>
    </div>

    <button
      onclick={toggle}
      class="p-2 rounded-full text-[var(--color-on-surface-variant)]
        hover:text-[var(--color-primary)] hover:bg-[var(--color-surface-container-high)]/50 transition-colors"
    >
      <span class="material-symbols-outlined text-[20px]">
        {isDark() ? 'light_mode' : 'dark_mode'}
      </span>
    </button>
  </div>
</header>
