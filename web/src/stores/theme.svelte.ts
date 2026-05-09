let dark = $state(
  typeof window !== 'undefined'
    ? localStorage.getItem('episteme-theme') !== 'light'
    : true,
);

export function isDark(): boolean {
  return dark;
}

export function toggle() {
  dark = !dark;
  localStorage.setItem('episteme-theme', dark ? 'dark' : 'light');
  applyClass();
}

export function initTheme() {
  applyClass();
}

function applyClass() {
  document.documentElement.classList.toggle('dark', dark);
}
