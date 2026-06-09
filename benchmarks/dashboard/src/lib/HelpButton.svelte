<script>
  import HelpModal from './HelpModal.svelte';
  import { i18n } from './i18n.svelte';

  /** @type {{ help: { en: { title: string; lines: string[] }; ko: { title: string; lines: string[] } } }} */
  let { help } = $props();
  let open = $state(false);

  let content = $derived(help[i18n.locale]);
</script>

<button class="help-btn" onclick={() => (open = true)} title="Help">?</button>
<HelpModal title={content.title} lines={content.lines} bind:open onclose={() => (open = false)} />

<style>
  .help-btn {
    width: 20px; height: 20px;
    border-radius: 50%;
    border: 1px solid #30363d;
    background: #161b22;
    color: #8b949e;
    font-size: 0.7rem;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    flex-shrink: 0;
    transition: border-color 0.15s, color 0.15s;
  }
  .help-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
