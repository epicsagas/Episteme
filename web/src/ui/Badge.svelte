<script lang="ts">
  import { ENTITY_TYPE_COLORS, ENTITY_TYPE_ICONS } from '../api/types.ts';
  import type { EntityType } from '../api/types.ts';

  let {
    type,
    label,
    small = false,
  }: {
    type: EntityType;
    label?: string;
    small?: boolean;
  } = $props();
</script>

<span
  class="inline-flex items-center gap-1 rounded-full font-bold
    {small ? 'px-2 py-0.5 text-[10px]' : 'px-3 py-1 text-[11px]'}
    border"
  style="
    background: color-mix(in srgb, {ENTITY_TYPE_COLORS[type]} 10%, transparent);
    color: {ENTITY_TYPE_COLORS[type]};
    border-color: color-mix(in srgb, {ENTITY_TYPE_COLORS[type]} 30%, transparent);
  "
>
  <span class="material-symbols-outlined {small ? 'text-[12px]' : 'text-[14px]'}">{ENTITY_TYPE_ICONS[type]}</span>
  {label ?? type}
</span>
