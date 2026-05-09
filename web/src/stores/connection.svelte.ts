export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected';

let status: ConnectionStatus = $state('disconnected');
let baseUrl: string = $state('http://localhost:8000');

export function getStatus(): ConnectionStatus {
  return status;
}

export function getBaseUrl(): string {
  return baseUrl;
}

export function setBaseUrl(url: string) {
  baseUrl = url;
}

export async function checkHealth(): Promise<boolean> {
  status = 'connecting';
  try {
    const res = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(3000) });
    if (res.ok) {
      status = 'connected';
      return true;
    }
  } catch {
    // server not reachable
  }
  status = 'disconnected';
  return false;
}
