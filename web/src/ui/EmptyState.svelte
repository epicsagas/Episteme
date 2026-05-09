<script lang="ts">
  let {
    icon = 'search_off',
    message = 'No data found',
  }: {
    icon?: string;
    message?: string;
  } = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 text-[var(--color-on-surface-variant)]">
  <span class="material-symbols-outlined text-4xl mb-3 opacity-40">{icon}</span>
  <p class="text-sm">{message}</p>
</div>
