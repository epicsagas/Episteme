<script>
  import QueryModal from './QueryModal.svelte';

  /** @type {{ perQuery: Array<{ query: string; top_ids: string[]; 'hit@1': number; 'hit@3': number; 'hit@5': number; 'rr@5': number; 'ndcg@5': number }> }} */
  let { perQuery } = $props();

  let selected = $state(null);
  let sortKey = $state(null);   // 'hit@1' | 'hit@3' | 'hit@5' | 'rr@5' | 'ndcg@5'
  let sortAsc = $state(true);
  let failuresOnly = $state(false);

  function rowClass(row) {
    if (row['hit@1'] === 0) return 'row-red';
    if (row['rr@5'] < 1.0) return 'row-yellow';
    return '';
  }

  function toggleSort(key) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = false; // default: descending (worst first)
    }
  }

  function sortIcon(key) {
    if (sortKey !== key) return '↕';
    return sortAsc ? '↑' : '↓';
  }

  let filtered = $derived(
    failuresOnly ? perQuery.filter((r) => r['hit@1'] === 0) : perQuery
  );

  let sorted = $derived.by(() => {
    if (!sortKey) return filtered;
    return [...filtered].sort((a, b) => {
      const av = a[sortKey], bv = b[sortKey];
      return sortAsc ? av - bv : bv - av;
    });
  });
</script>

<div class="table-toolbar">
  <label class="filter-toggle">
    <input type="checkbox" bind:checked={failuresOnly} />
    Show failures only
  </label>
</div>

<div class="table-wrap">
  <table>
    <thead>
      <tr>
        <th>Query</th>
        <th class="sortable" onclick={() => toggleSort('hit@1')}>hit@1 {sortIcon('hit@1')}</th>
        <th class="sortable" onclick={() => toggleSort('hit@3')}>hit@3 {sortIcon('hit@3')}</th>
        <th class="sortable" onclick={() => toggleSort('hit@5')}>hit@5 {sortIcon('hit@5')}</th>
        <th class="sortable" onclick={() => toggleSort('rr@5')}>RR {sortIcon('rr@5')}</th>
        <th class="sortable" onclick={() => toggleSort('ndcg@5')}>NDCG {sortIcon('ndcg@5')}</th>
      </tr>
    </thead>
    <tbody>
      {#each sorted as row}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_interactive_element_to_noninteractive_role -->
        <tr class={rowClass(row)} onclick={() => (selected = row)}>
          <td class="query-cell">{row.query}</td>
          <td class="center">{row['hit@1'] === 1 ? '✓' : '✗'}</td>
          <td class="center">{row['hit@3'] === 1 ? '✓' : '✗'}</td>
          <td class="center">{row['hit@5'] === 1 ? '✓' : '✗'}</td>
          <td class="num">{row['rr@5'].toFixed(3)}</td>
          <td class="num">{row['ndcg@5'].toFixed(3)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<QueryModal query={selected} onclose={() => (selected = null)} />

<style>
  .table-toolbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-top: 0.5rem;
  }

  .filter-toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.82rem;
    color: #8b949e;
    cursor: pointer;
  }

  .filter-toggle input {
    accent-color: var(--accent);
  }

  .table-wrap {
    overflow-x: auto;
    border: 1px solid #21262d;
    border-radius: 8px;
    margin-top: 1rem;
  }

  table { width: 100%; border-collapse: collapse; font-size: 0.83rem; }

  thead th {
    background: #161b22;
    color: #8b949e;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.75rem;
    padding: 0.6rem 0.75rem;
    text-align: left;
    position: sticky;
    top: 0;
  }

  .sortable {
    cursor: pointer;
    user-select: none;
    transition: color 0.15s;
  }
  .sortable:hover { color: var(--accent); }

  tbody tr { cursor: pointer; border-top: 1px solid #21262d; transition: background 0.1s; }
  tbody tr:hover { background: #161b22; }

  td { padding: 0.55rem 0.75rem; color: #c9d1d9; }

  .query-cell { max-width: 200px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .center { text-align: center; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }

  .row-red { background: #3d0c0c55; }
  .row-red:hover { background: #3d0c0c88; }
  .row-yellow { background: #3d2e0055; }
  .row-yellow:hover { background: #3d2e0088; }
</style>
