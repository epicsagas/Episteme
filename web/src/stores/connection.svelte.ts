export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected';

let status: ConnectionStatus = $state('disconnected');
let baseUrl: string = $state('http://localhost:8000');
let webPort: number = $state(8080);

let readyResolve: (() => void) | null = null;
let readyPromise = new Promise<void>((resolve) => {
  readyResolve = resolve;
});

export function getStatus(): ConnectionStatus {
  return status;
}

export function getBaseUrl(): string {
  return baseUrl;
}

export function getWebUrl(): string {
  const url = new URL(baseUrl);
  url.port = String(webPort);
  return url.origin;
}

export function setBaseUrl(url: string) {
  baseUrl = url;
}

export function setWebPort(port: number) {
  webPort = port;
  notifyReady();
}

export function waitForReady(): Promise<void> {
  return Promise.race([
    readyPromise,
    new Promise<void>((resolve) => setTimeout(resolve, 8000)),
  ]);
}

function notifyReady() {
  if (readyResolve) {
    readyResolve();
    readyResolve = null;
  }
}

/** Resolve the ready promise immediately (used in non-Tauri mode). */
export function markReady() {
  notifyReady();
}

export async function checkHealth(maxRetries = 3): Promise<boolean> {
  status = 'connecting';
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const res = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(3000) });
      if (res.ok) {
        status = 'connected';
        return true;
      }
    } catch {
      // server not reachable yet
    }
    if (attempt < maxRetries - 1) {
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
  status = 'disconnected';
  return false;
}
