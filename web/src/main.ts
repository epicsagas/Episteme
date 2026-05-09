import { mount } from 'svelte';
import App from './App.svelte';
import { initTheme } from './stores/theme.svelte.ts';

initTheme();

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
