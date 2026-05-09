<script lang="ts">
  import { ENTITY_TYPE_COLORS } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';

  const items: Array<{ type: EntityType; label: string }> = [
    { type: 'pattern', label: 'Design Patterns' },
    { type: 'refactoring', label: 'Refactorings' },
    { type: 'law', label: 'Laws & Principles' },
    { type: 'smell', label: 'Code Smells' },
    { type: 'insight', label: 'Insights' },
  ];

  const relItems = [
    { color: '#42a5f5', label: 'enforces' },
    { color: '#64b5f6', label: 'enforced_by' },
    { color: '#ef5350', label: 'violates' },
    { color: '#e57373', label: 'violated_by' },
    { color: '#66bb6a', label: 'solves' },
    { color: '#78909c', label: 'related_to' },
  ];
</script>

<div class="absolute right-6 top-6 z-30">
  <div class="glass-panel p-4 w-48">
    <h3 class="text-[11px] font-bold uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-3">Node Types</h3>
    <div class="space-y-2">
      {#each items as item}
        <div class="flex items-center gap-2">
          <span class="w-3 h-3 rounded-full shrink-0" style="background: {ENTITY_TYPE_COLORS[item.type]}"></span>
          <span class="text-xs text-[var(--color-on-surface)]">{item.label}</span>
        </div>
      {/each}
    </div>
    <div class="mt-3 pt-3 border-t border-[var(--color-outline-variant)]">
      <h3 class="text-[11px] font-bold uppercase tracking-wider text-[var(--color-on-surface-variant)] mb-2">Relations</h3>
      <div class="space-y-1.5">
        {#each relItems as rel}
          <div class="flex items-center gap-2">
            <div class="w-5 h-0.5 shrink-0" style="background: {rel.color}"></div>
            <span class="text-[10px] text-[var(--color-on-surface-variant)]">{rel.label}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
