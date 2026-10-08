import { ArrowUpFromLine, CircleAlert, FileText, History, LayoutDashboard, Library, Lock, Moon, PencilLine, RefreshCw, Settings as SettingsIcon, Stethoscope, Sun, X } from 'lucide-solid';
import { For, type JSX, Match, Show, Switch, createResource, createSignal } from 'solid-js';
import { api } from './api';
import { Spinner } from './components';
import { route, type Route } from './router';
import { dismiss, toasts } from './toast';
import Doctor from './views/Doctor';
import History2 from './views/History';
import Overview from './views/Overview';
import Run from './views/Run';
import Settings from './views/Settings';
import Vault from './views/Vault';

const nav: { route: Route; path: string; label: string; icon: JSX.Element }[] = [
  { route: '', path: '/', label: 'Overview', icon: <LayoutDashboard size={18} /> },
  { route: 'skills', path: '/skills', label: 'Skills', icon: <Library size={18} /> },
  { route: 'sync', path: '/sync', label: 'Sync', icon: <RefreshCw size={18} class="turn-on-hover" /> },
  { route: 'push', path: '/push', label: 'Push', icon: <ArrowUpFromLine size={18} class="lift-on-hover" /> },
  { route: 'history', path: '/history', label: 'History', icon: <History size={18} /> },
  { route: 'doctor', path: '/doctor', label: 'Doctor', icon: <Stethoscope size={18} /> },
  { route: 'settings', path: '/settings', label: 'Settings', icon: <SettingsIcon size={18} /> },
];

function ThemeButton() {
  const [dark, setDark] = createSignal(
    document.documentElement.getAttribute('data-theme') === 'dark'
      || (document.documentElement.getAttribute('data-theme') === null && window.matchMedia('(prefers-color-scheme: dark)').matches),
  );
  function flip() {
    const next = !dark();
    setDark(next);
    document.documentElement.setAttribute('data-theme', next ? 'dark' : 'light');
    try { localStorage.setItem('skillmirror-theme', next ? 'dark' : 'light'); } catch { /* the choice just does not stick */ }
  }
  return (
    <button class="button button-quiet" onClick={flip} aria-label={dark() ? 'Switch to the light theme' : 'Switch to the dark theme'}>
      <Show when={dark()} fallback={<Moon size={16} class="swing" />}><Sun size={16} class="swing" /></Show>
      {dark() ? 'Light' : 'Dark'}
    </button>
  );
}

function Toasts() {
  return (
    <div class="toasts" role="status" aria-live="polite">
      <For each={toasts()}>{(t) => (
        <div class={`toast toast-${t.tone}`}>
          <CircleAlert size={18} /><p class="pre">{t.text}</p>
          <button class="button-quiet icon-only" onClick={() => dismiss(t.id)} aria-label="Dismiss"><X size={16} /></button>
        </div>
      )}</For>
    </div>
  );
}

export default function App() {
  const [session] = createResource(api.session);
  const write = () => session()?.allow_write ?? false;
  return (
    <div class="shell">
      <aside class="sidebar">
        <div class="brand"><span class="logo" aria-hidden="true" /><span>skillmirror</span></div>
        <nav aria-label="Views">
          <For each={nav}>{(item) => (
            <a data-link href={item.path} class={route() === item.route ? 'nav-link on' : 'nav-link'} aria-current={route() === item.route ? 'page' : undefined}>
              {item.icon}{item.label}
            </a>
          )}</For>
          <a href="/report" class="nav-link"><FileText size={18} />Report</a>
        </nav>
        <div class="side-foot">
          <Show when={session()}>{(s) => (
            <span class={write() ? 'mode mode-write' : 'mode'} title={write() ? 'Sync, push and undo can be run from these pages' : 'Start with --allow-write to run sync, push and undo from here'}>
              {write() ? <PencilLine size={14} /> : <Lock size={14} />}{write() ? 'Changes allowed' : 'Read-only'}
              <span class="muted small"> v{s().version}</span>
            </span>
          )}</Show>
          <ThemeButton />
        </div>
      </aside>
      <main>
        <Show when={!session.error} fallback={<p class="bad pre">{(session.error as Error).message}</p>}>
          <Show when={session()} fallback={<div class="loading"><Spinner /></div>}>
            <Switch>
              <Match when={route() === ''}><Overview /></Match>
              <Match when={route() === 'skills'}><Vault /></Match>
              <Match when={route() === 'sync'}><Run kind="sync" write={write()} /></Match>
              <Match when={route() === 'push'}><Run kind="push" write={write()} /></Match>
              <Match when={route() === 'history'}><History2 write={write()} /></Match>
              <Match when={route() === 'doctor'}><Doctor /></Match>
              <Match when={route() === 'settings'}><Settings /></Match>
            </Switch>
          </Show>
        </Show>
      </main>
      <Toasts />
    </div>
  );
}
