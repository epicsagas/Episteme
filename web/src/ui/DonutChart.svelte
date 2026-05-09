<script lang="ts">
  type Segment = { label: string; value: number; color: string };

  let {
    segments,
    size = 192,
    strokeWidth = 14,
    centerLabel,
    centerValue,
  }: {
    segments: Segment[];
    size?: number;
    strokeWidth?: number;
    centerLabel?: string;
    centerValue?: string;
  } = $props();

  let radius = $derived((size - strokeWidth) / 2);
  let circumference = $derived(2 * Math.PI * radius);
  let total = $derived(segments.reduce((sum, s) => sum + s.value, 0));

  let offsets = $derived.by(() => {
    let offset = 0;
    return segments.map((s) => {
      const dash = (s.value / total) * circumference;
      const gap = circumference - dash;
      const entry = { dash, gap, offset };
      offset += dash;
      return entry;
    });
  });
</script>

<div class="relative inline-flex items-center justify-center" style="width: {size}px; height: {size}px;">
  <svg width={size} height={size} class="transform -rotate-90">
    <!-- background track -->
    <circle
      cx={size / 2} cy={size / 2} r={radius}
      fill="transparent"
      stroke="var(--color-surface-container-high)"
      stroke-width={strokeWidth}
    />
    {#each segments as seg, i}
      <circle
        cx={size / 2} cy={size / 2} r={radius}
        fill="transparent"
        stroke={seg.color}
        stroke-width={strokeWidth}
        stroke-dasharray="{offsets[i].dash} {offsets[i].gap}"
        stroke-dashoffset={-offsets[i].offset}
        stroke-linecap="round"
      />
    {/each}
  </svg>
  <div class="absolute inset-0 flex flex-col items-center justify-center">
    {#if centerValue}
      <span class="text-2xl font-bold text-[var(--color-on-surface)]">{centerValue}</span>
    {/if}
    {#if centerLabel}
      <span class="text-[10px] uppercase font-semibold text-[var(--color-on-surface-variant)]">{centerLabel}</span>
    {/if}
  </div>
</div>
