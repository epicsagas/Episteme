<script lang="ts">
  import './app.css';
  import Sidebar from './layout/Sidebar.svelte';
  import Topbar from './layout/Topbar.svelte';
  import Dashboard from './pages/Dashboard.svelte';
  import Explorer from './pages/Explorer.svelte';
  import EntityDetail from './pages/EntityDetail.svelte';
  import Ontology from './pages/Ontology.svelte';
  import { getCurrentRoute, navigate } from './router/index.svelte.ts';
  import { checkHealth, setBaseUrl, setWebPort, markReady } from './stores/connection.svelte.ts';
  import { clearSelection } from './stores/graph.svelte.ts';
  import { onMount } from 'svelte';

  onMount(() => {
    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        await listen<{ api_port: number; web_port: number }>('backend-ready', (event) => {
          setBaseUrl(`http://localhost:${event.payload.api_port}`);
          setWebPort(event.payload.web_port);
        });
      } catch {
        // Not running in Tauri — use defaults and resolve ready immediately
        markReady();
      }
      await checkHealth();
    })();

    function handleKeydown(e: KeyboardEvent) {
      // Cmd/Ctrl+K -> focus search
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        document.getElementById('global-search')?.focus();
      }
      // Escape -> clear selection or go back
      if (e.key === 'Escape') {
        clearSelection();
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  let route = $derived(getCurrentRoute());
</script>

<div class="flex h-screen w-screen overflow-hidden bg-[var(--color-background)]">
  <Sidebar />
  <main class="flex-1 flex flex-col h-full overflow-hidden">
    <Topbar />
    <div class="flex-1 overflow-y-auto p-6">
      {#if route.page === 'dashboard'}
        <Dashboard />
      {:else if route.page === 'explorer'}
        <Explorer />
      {:else if route.page === 'entity'}
        <EntityDetail />
      {:else if route.page === 'ontology'}
        <Ontology />
      {/if}
    </div>
  </main>
</div>
