export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiGet<T>(baseUrl: string, path: string, signal?: AbortSignal): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${baseUrl}${path}`, { signal });
  } catch (e) {
    if (signal?.aborted) throw e;
    throw new ApiError(e instanceof TypeError ? 'Network error — server unreachable' : String(e), 0);
  }
  if (!res.ok) {
    throw new ApiError(`API error: ${res.status} ${res.statusText}`, res.status);
  }
  try {
    return await res.json();
  } catch {
    throw new ApiError('Invalid JSON response from server', res.status);
  }
}

export async function apiPost<TReq, TRes>(baseUrl: string, path: string, body: TReq, signal?: AbortSignal): Promise<TRes> {
  let res: Response;
  try {
    res = await fetch(`${baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      signal,
    });
  } catch (e) {
    if (signal?.aborted) throw e;
    throw new ApiError(e instanceof TypeError ? 'Network error — server unreachable' : String(e), 0);
  }
  if (!res.ok) {
    throw new ApiError(`API error: ${res.status} ${res.statusText}`, res.status);
  }
  try {
    return await res.json();
  } catch {
    throw new ApiError('Invalid JSON response from server', res.status);
  }
}
