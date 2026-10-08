import { createSignal } from 'solid-js';

export interface Toast { id: number; tone: 'bad' | 'ok'; text: string }

const [toasts, setToasts] = createSignal<Toast[]>([]);
export { toasts };

let next = 0;
/** Says something short, and takes it away again after a while. */
export function notify(text: string, tone: Toast['tone'] = 'bad'): void {
  const id = ++next;
  setToasts((list) => [...list, { id, tone, text }]);
  setTimeout(() => dismiss(id), 8000);
}

export function dismiss(id: number): void {
  setToasts((list) => list.filter((t) => t.id !== id));
}

/** The words of whatever was thrown. */
export function words(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
