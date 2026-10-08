import { CircleAlert, LoaderCircle, RefreshCw } from 'lucide-solid';
import { For, type JSX, Match, type Resource, Show, Switch, createSignal, onMount } from 'solid-js';
import { appear, countUp, reveal } from './motion';
import type { DiffFile } from './types';

/** A small coloured label. */
export function Chip(props: { tone?: 'ok' | 'warn' | 'bad' | 'muted' | 'accent'; children: JSX.Element; title?: string }) {
  return <span class={`chip chip-${props.tone ?? 'muted'}`} title={props.title}>{props.children}</span>;
}

/** A number that counts up when it first shows. */
export function Stat(props: { label: string; value: number; tone?: 'ok' | 'warn' | 'bad'; icon: JSX.Element }) {
  let value!: HTMLSpanElement;
  onMount(() => countUp(value, props.value));
  return (
    <div class={`stat stat-${props.tone ?? 'plain'}`}>
      <div class="stat-icon">{props.icon}</div>
      <div>
        <span class="stat-value" ref={value}>0</span>
        <span class="stat-label">{props.label}</span>
      </div>
    </div>
  );
}

/** The turning ring that says something is being worked on. */
export function Spinner(props: { label?: string }) {
  return <span class="spin-line"><LoaderCircle class="spin" size={18} />{props.label}</span>;
}

export function Empty(props: { icon: JSX.Element; title: string; children?: JSX.Element }) {
  return (
    <div class="empty" ref={appear}>
      <div class="empty-icon">{props.icon}</div>
      <h3>{props.title}</h3>
      <Show when={props.children}><p>{props.children}</p></Show>
    </div>
  );
}

/** A box that shows what a request answered: a spinner meanwhile, the reason and a retry when it failed. */
export function Loaded<T>(props: { of: Resource<T>; retry: () => void; children: (data: T) => JSX.Element }) {
  return (
    <Switch>
      <Match when={props.of.error}>
        <div class="failure" ref={appear}>
          <CircleAlert size={22} />
          <div>
            <h3>This could not be read</h3>
            <p class="pre">{props.of.error instanceof Error ? props.of.error.message : String(props.of.error)}</p>
            <button class="button" onClick={props.retry}><RefreshCw size={16} />Try again</button>
          </div>
        </div>
      </Match>
      <Match when={props.of.loading && props.of() === undefined}>
        <div class="loading"><Spinner label="Looking at the disk…" /></div>
      </Match>
      <Match when={props.of() !== undefined}>{props.children(props.of() as T)}</Match>
    </Switch>
  );
}

/** One file of a plan: its kind and counts, and the lines when there are any. */
export function DiffView(props: { file: DiffFile }) {
  const [open, setOpen] = createSignal(false);
  const lines = () => props.file.text.split('\n');
  const kind = (line: string) =>
    line.startsWith('+') && !line.startsWith('+++') ? 'add'
      : line.startsWith('-') && !line.startsWith('---') ? 'del'
      : line.startsWith('@@') ? 'hunk' : '';
  return (
    <div class="diff-file">
      <button class="linklike" onClick={() => setOpen(!open())} aria-expanded={open()} disabled={!props.file.text}>
        <span class="path">{props.file.path}</span>
        <span class="muted"> {props.file.kind}</span>
        <Show when={props.file.added || props.file.removed}>
          <span class="add-count"> +{props.file.added}</span><span class="del-count"> −{props.file.removed}</span>
        </Show>
      </button>
      <Show when={props.file.note}><div class="muted note-line">{props.file.note}</div></Show>
      <Show when={open() && props.file.text}>
        <pre class="diff" ref={appear}>
          <For each={lines()}>{(line) => <span class={kind(line)}>{line}{'\n'}</span>}</For>
        </pre>
      </Show>
    </div>
  );
}

/** The children appear one after another when the view first shows. */
export function Cascade(props: { class?: string; children: JSX.Element }) {
  return <div class={props.class} ref={(el) => reveal(el)}>{props.children}</div>;
}
