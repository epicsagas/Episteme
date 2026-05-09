<script lang="ts">
  let {
    placeholder = 'Search...',
    onsearch,
  }: {
    placeholder?: string;
    onsearch: (query: string) => void;
  } = $props();

  let value = $state('');
  let timer: ReturnType<typeof setTimeout>;

  function handleInput() {
    clearTimeout(timer);
    timer = setTimeout(() => onsearch(value), 300);
  }
</script>

<div class="relative">
  <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2
    text-[var(--color-on-surface-variant)] pointer-events-none text-[20px]">search</span>
  <input
    type="text"
    bind:value
    oninput={handleInput}
    {placeholder}
    class="w-full bg-[var(--color-surface-container-low)] border border-[var(--color-outline-variant)]
      rounded-lg pl-10 pr-4 py-2 text-sm outline-none
      focus:ring-1 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
      text-[var(--color-on-surface)] placeholder:text-[var(--color-outline)]"
  />
</div>
