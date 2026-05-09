export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected';

const READY_TIMEOUT_MS = 8_000;
const HEALTH_CHECK_TIMEOUT_MS = 3_000;
const HEALTH_MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1_000;

let status: ConnectionStatus = $state('disconnected');
let baseUrl: string = $state('http://localhost:8000');
let webPort: number = $state(8080);

let readyResolve: ((value: 'ready' | 'timeout') => void) | null = null;
let readyPromise = new Promise<'ready' | 'timeout'>((resolve) => {
  readyResolve = resolve;
});

export function getStatus(): ConnectionStatus {
  return status;
}

export function getBaseUrl(): string {
  return baseUrl;
}

export function getWebUrl(): string {
  return `${baseUrl.split(':').slice(0, 2).join(':')}:${webPort}`;
}

export function setBaseUrl(url: string) {
  baseUrl = url;
}

export function setWebPort(port: number) {
  webPort = port;
  notifyReady('ready');
}

export async function waitForReady(): Promise<'ready' | 'timeout'> {
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
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const res = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(HEALTH_CHECK_TIMEOUT_MS) });
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
