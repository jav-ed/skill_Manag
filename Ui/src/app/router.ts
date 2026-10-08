import { createSignal } from 'solid-js';

export const routes = ['', 'skills', 'sync', 'push', 'history', 'doctor', 'settings'] as const;
export type Route = (typeof routes)[number];

function current(): { route: Route; query: URLSearchParams } {
  const name = location.pathname.replace(/^\/+|\/+$/g, '');
  const route = (routes as readonly string[]).includes(name) ? (name as Route) : '';
  return { route, query: new URLSearchParams(location.search) };
}

const [state, setState] = createSignal(current());
export const route = () => state().route;
export const query = () => state().query;

/** Moves to another view without loading the page again. */
export function navigate(to: string): void {
  history.pushState(null, '', to);
  setState(current());
  window.scrollTo({ top: 0 });
}

window.addEventListener('popstate', () => setState(current()));

/** Links to our own views are followed in place; everything else is left to the browser. */
document.addEventListener('click', (event) => {
  if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  const link = (event.target as Element | null)?.closest('a[data-link]');
  if (!(link instanceof HTMLAnchorElement)) return;
  event.preventDefault();
  navigate(link.getAttribute('href') ?? '/');
});
