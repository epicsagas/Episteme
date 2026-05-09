import type { HealthResponse } from './types.ts';

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiGet<T>(baseUrl: string, path: string): Promise<T> {
  const res = await fetch(`${baseUrl}${path}`);
  if (!res.ok) {
    throw new ApiError(`API error: ${res.status} ${res.statusText}`, res.status);
  }
  return res.json();
}

export async function apiPost<TReq, TRes>(baseUrl: string, path: string, body: TReq): Promise<TRes> {
  const res = await fetch(`${baseUrl}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new ApiError(`API error: ${res.status} ${res.statusText}`, res.status);
  }
  return res.json();
}
