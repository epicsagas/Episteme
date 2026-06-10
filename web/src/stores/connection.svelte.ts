export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected';

const READY_TIMEOUT_MS = 8_000;
const HEALTH_CHECK_TIMEOUT_MS = 3_000;
const HEALTH_MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1_000;

let status: ConnectionStatus = $state('disconnected');

let readyResolve: ((value: 'ready' | 'timeout') => void) | null = null;
let readyPromise = new Promise<'ready' | 'timeout'>((resolve) => {
  readyResolve = resolve;
});

export function getStatus(): ConnectionStatus {
  return status;
}

/**
 * Base URL for the API server (entity, search, health, stats).
 * In dev mode (Vite port 5173), goes through /api/v1 proxy → localhost:58302.
 * In production (Tauri), set dynamically via setBaseUrl().
 */
export function getBaseUrl(): string {
  if (typeof window !== 'undefined' && window.location.port === '5173') {
    return `${window.location.origin}/api/v1`;
  }
  // Production fallback — Tauri sets this via setBaseUrl()
  return 'http://localhost:58302';
}

/**
 * Base URL for the Web viewer server (graph data, tree, schema).
 * In dev mode (Vite port 5173), goes through /api/web proxy → localhost:8080.
 * In production (Tauri), set dynamically via setWebPort().
 */
export function getWebUrl(): string {
  if (typeof window !== 'undefined' && window.location.port === '5173') {
    return `${window.location.origin}/api/web`;
  }
  // Production fallback
  return 'http://localhost:8080';
}

// Tauri dynamic config — called when running inside Tauri shell
let _baseUrlOverride: string | null = null;
let _webPortOverride: number | null = null;

export function setBaseUrl(url: string) {
  _baseUrlOverride = url;
}

export function setWebPort(port: number) {
  _webPortOverride = port;
  notifyReady('ready');
}

export async function waitForReady(): Promise<'ready' | 'timeout'> {
  // In dev mode (Vite), skip waiting — proxies are available immediately
  if (typeof window !== 'undefined' && window.location.port === '5173') {
    // Just do a health check to verify the backend is up
    const healthy = await checkHealth();
    return healthy ? 'ready' : 'timeout';
  }
  // In Tauri mode, wait for the backend-ready event
  const result = await Promise.race([
    readyPromise,
    new Promise<'timeout'>((resolve) => setTimeout(() => resolve('timeout'), READY_TIMEOUT_MS)),
  ]);
  return result;
}

function notifyReady(value: 'ready' | 'timeout') {
  if (readyResolve) {
    readyResolve(value);
    readyResolve = null;
  }
}

export function markReady() {
  notifyReady('ready');
}

export async function checkHealth(maxRetries = HEALTH_MAX_RETRIES): Promise<boolean> {
  status = 'connecting';
  const base = getBaseUrl();
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const res = await fetch(`${base}/health`, { signal: AbortSignal.timeout(HEALTH_CHECK_TIMEOUT_MS) });
      if (res.ok) {
        status = 'connected';
        return true;
      }
    } catch {
      // server not reachable yet
    }
    if (attempt < maxRetries - 1) {
      await new Promise((r) => setTimeout(r, RETRY_DELAY_MS));
    }
  }
  status = 'disconnected';
  return false;
}
