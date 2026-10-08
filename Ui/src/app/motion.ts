import { animate, stagger } from 'motion';
import { onMount } from 'solid-js';

/** People who asked their system for less movement get none. */
export const calm = (): boolean => window.matchMedia('(prefers-reduced-motion: reduce)').matches;

/** Fades and lifts the children of an element in, one after another. Call it from `ref`. */
export function reveal(container: HTMLElement, selector = ':scope > *'): void {
  onMount(() => {
    if (calm()) return;
    const items = container.querySelectorAll(selector);
    if (items.length === 0) return;
    animate(items, { opacity: [0, 1], transform: ['translateY(10px)', 'translateY(0px)'] }, { duration: 0.35, delay: stagger(0.04, { startDelay: 0.02 }), ease: 'easeOut' });
  });
}

/** A single element appearing: used for panels that open after a click. */
export function appear(element: HTMLElement): void {
  if (calm()) return;
  animate(element, { opacity: [0, 1], transform: ['translateY(8px) scale(0.99)', 'translateY(0px) scale(1)'] }, { duration: 0.28, ease: 'easeOut' });
}

/** Counts a number up to its value. */
export function countUp(element: HTMLElement, to: number): void {
  if (calm() || to === 0) { element.textContent = String(to); return; }
  animate(0, to, { duration: 0.7, ease: 'easeOut', onUpdate: (value) => { element.textContent = String(Math.round(value)); } });
}

/** A bar that moves smoothly to a width given as a fraction of its track. */
export function growTo(bar: HTMLElement, fraction: number): void {
  const width = `${Math.max(0, Math.min(1, fraction)) * 100}%`;
  if (calm()) { bar.style.width = width; return; }
  animate(bar, { width }, { duration: 0.25, ease: 'easeOut' });
}
