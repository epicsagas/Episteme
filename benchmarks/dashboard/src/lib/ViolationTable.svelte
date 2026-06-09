<script>
  /** @type {{ perQuery: Array<{ query: string; category: string; must_not_contain: string[]; top_ids: string[]; 'fp@1': number; 'fp@3': number; 'fp@5': number; violations: string[] }> }} */
  let { perQuery } = $props();
</script>

<div class="table-wrap">
  <table>
    <thead>
      <tr>
        <th>Query</th>
        <th>Category</th>
        <th>FP@1</th>
        <th>FP@3</th>
        <th>FP@5</th>
        <th>Violations</th>
      </tr>
    </thead>
    <tbody>
      {#each perQuery as row}
        <tr class:violated={row.violations?.length > 0}>
          <td class="query-cell">{row.query}</td>
          <td class="cat">{row.category}</td>
          <td class="center">{row['fp@1']}</td>
          <td class="center">{row['fp@3']}</td>
          <td class="center">{row['fp@5']}</td>
          <td class="chips">
            {#each row.violations ?? [] as v}
              <span class="chip">{v}</span>
            {/each}
            {#if !row.violations?.length}
              <span class="clean">—</span>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
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

  tbody tr { border-top: 1px solid #21262d; transition: background 0.1s; }
  tbody tr:hover { background: #161b22; }

  td { padding: 0.55rem 0.75rem; color: #c9d1d9; }

  .query-cell { max-width: 200px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cat { color: #8b949e; font-size: 0.78rem; }
  .center { text-align: center; }

  .violated { background: #3d0c0c55; }
  .violated:hover { background: #3d0c0c88; }

  .chips { display: flex; flex-wrap: wrap; gap: 0.3rem; }

  .chip {
    background: #3d0c0c;
    border: 1px solid var(--red);
    border-radius: 4px;
    padding: 0.15rem 0.4rem;
    font-size: 0.75rem;
    color: var(--red);
    font-family: monospace;
  }

  .clean { color: #8b949e; }
</style>
