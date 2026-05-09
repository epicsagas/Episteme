<script lang="ts">
  let { class: className = '' }: { class?: string } = $props();
</script>

<div class="animate-pulse rounded-lg bg-[var(--color-surface-container-high)] {className}"></div>
