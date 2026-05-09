<script lang="ts">
  import { isDark, toggle } from '../stores/theme.svelte.ts';
  import { getStatus } from '../stores/connection.svelte.ts';
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
        placeholder="Search entities... (⌘K)"
        class="bg-[var(--color-surface-container-low)] border border-[var(--color-outline-variant)]
          rounded-full pl-10 pr-10 py-1.5 text-sm w-80 outline-none
          focus:ring-1 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
          text-[var(--color-on-surface)] placeholder:text-[var(--color-outline)]"
      />
      <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-mono
        text-[var(--color-outline)] border border-[var(--color-outline-variant)] px-1 rounded">⌘K</span>
    </div>
  </div>

  <div class="flex items-center gap-4">
    <div class="flex items-center gap-1">
      <button class="p-2 rounded-full text-[var(--color-on-surface-variant)]
        hover:text-[var(--color-primary)] hover:bg-[var(--color-surface-container-high)]/50 transition-colors">
        <span class="material-symbols-outlined text-[20px]">history</span>
      </button>
      <button class="p-2 rounded-full text-[var(--color-on-surface-variant)] relative
        hover:text-[var(--color-primary)] hover:bg-[var(--color-surface-container-high)]/50 transition-colors">
        <span class="material-symbols-outlined text-[20px]">notifications</span>
      </button>
    </div>

    <div class="flex items-center gap-1 border-l border-[var(--color-outline-variant)] pl-4">
      <div class="w-2 h-2 rounded-full
        {getStatus() === 'connected' ? 'bg-[var(--color-rel-solves)]' : getStatus() === 'connecting' ? 'bg-[var(--color-law)]' : 'bg-[var(--color-error)]'}">
      </div>
      <span class="text-[10px] text-[var(--color-on-surface-variant)]">
        {getStatus() === 'connected' ? 'Connected' : getStatus() === 'connecting' ? 'Connecting...' : 'Offline'}
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
