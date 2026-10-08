import type { Doctor, Finished, History, JobView, Overview, Plan, Session, Settings, Skills, UndoPlan, Vault } from './types';

/** The server refused or failed; `message` is a sentence for the person. */
export class ApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

async function call<T>(method: 'GET' | 'POST', path: string, body?: unknown): Promise<T> {
  const init: RequestInit = { method, credentials: 'same-origin', headers: {} };
  if (method === 'POST') {
    // A change is JSON with our own header; the server refuses anything else, so another site cannot send one.
    init.headers = { 'Content-Type': 'application/json', 'X-Skillmirror': '1' };
    init.body = JSON.stringify(body ?? {});
  }
  let response: Response;
  try {
    response = await fetch(path, init);
  } catch {
    throw new ApiError('The server cannot be reached. Is `skillmirror web` still running?', 0);
  }
  let data: unknown = null;
  try { data = await response.json(); } catch { /* an empty body */ }
  if (!response.ok) {
    const message = (data as { error?: string } | null)?.error;
    throw new ApiError(message ?? `${response.status} ${response.statusText}`, response.status);
  }
  return data as T;
}

export const api = {
  session: () => call<Session>('GET', '/api/session'),
  overview: () => call<Overview>('GET', '/api/overview'),
  skills: (kind: 'sync' | 'push') => call<Skills>('GET', `/api/skills/${kind}`),
  vault: () => call<Vault>('GET', '/api/vault'),
  history: () => call<History>('GET', '/api/history'),
  doctor: () => call<Doctor>('GET', '/api/doctor'),
  settings: () => call<Settings>('GET', '/api/settings'),
  rescan: () => call<{ at: string; projects: number }>('POST', '/api/rescan'),
  plan: (kind: 'sync' | 'push', skills: string[], project: string | null) =>
    call<Plan>('POST', '/api/plan', { kind, skills, project }),
  undoPlan: (run: string) => call<UndoPlan>('POST', '/api/undo-plan', { run }),
  apply: (plan: string) => call<{ job: string }>('POST', '/api/apply', { plan }),
  job: (id: string) => call<JobView>('GET', `/api/job/${encodeURIComponent(id)}`),
};

/** Follows a job until it ends, reporting how far it has come. */
export async function watch(job: string, onProgress: (done: number, total: number) => void): Promise<Finished> {
  for (;;) {
    const view = await api.job(job);
    if (view.state === 'running') {
      onProgress(view.done, view.total);
      await new Promise((resolve) => setTimeout(resolve, 200));
    } else if (view.state === 'done') {
      return view;
    } else {
      throw new ApiError(view.message, 500);
    }
  }
}
