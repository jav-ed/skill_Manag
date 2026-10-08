import { ArrowRight, CircleCheck, History as HistoryIcon, TriangleAlert, X } from 'lucide-solid';
import { For, type JSX, Match, Show, Switch, createSignal } from 'solid-js';
import { api, watch } from './api';
import { Chip, Spinner } from './components';
import { appear, growTo } from './motion';
import { navigate } from './router';
import { notify, words } from './toast';
import type { Finished } from './types';

type Phase = { name: 'review' } | { name: 'working'; done: number; total: number } | { name: 'done'; result: Finished };

/** The three steps of every change: look at the plan, watch it run, read what happened. */
export function Flow(props: {
  planId: string;
  review: JSX.Element;
  apply: string;
  danger?: boolean;
  disabled?: boolean;
  onClose: () => void;
  /** The change was written; whatever shows the old state should look again. */
  onApplied?: () => void;
}) {
  const [phase, setPhase] = createSignal<Phase>({ name: 'review' });
  let bar: HTMLDivElement | undefined;

  async function run() {
    setPhase({ name: 'working', done: 0, total: 0 });
    try {
      const { job } = await api.apply(props.planId);
      const result = await watch(job, (done, total) => {
        setPhase({ name: 'working', done, total });
        if (bar) growTo(bar, total ? done / total : 0);
      });
      setPhase({ name: 'done', result });
      props.onApplied?.();
    } catch (error) {
      notify(words(error));
      setPhase({ name: 'review' });
    }
  }

  return (
    <section class="panel" ref={appear} aria-live="polite">
      <Switch>
        <Match when={phase().name === 'review'}>
          {props.review}
          <div class="actions">
            <button class={`button ${props.danger ? 'button-danger' : 'button-primary'}`} disabled={props.disabled} onClick={run}>
              <ArrowRight size={16} />{props.apply}
            </button>
            <button class="button" onClick={props.onClose}><X size={16} />Cancel</button>
          </div>
          <p class="muted small">Folders that are replaced are saved first; History can bring them back.</p>
        </Match>
        <Match when={phase().name === 'working'}>
          <h3><Spinner label="Working…" /></h3>
          <div class="track"><div class="bar" ref={bar} /></div>
          <Show when={(phase() as { total: number }).total > 0}>
            <p class="muted small">{(phase() as { done: number }).done} of {(phase() as { total: number }).total} folders</p>
          </Show>
        </Match>
        <Match when={phase().name === 'done'}>
          <Result result={(phase() as { result: Finished }).result} onClose={props.onClose} />
        </Match>
      </Switch>
    </section>
  );
}

const outcomeTone = (outcome: string) =>
  outcome === 'failed' ? 'bad' : outcome === 'unchanged' || outcome === 'gone' ? 'muted' : outcome === 'created' || outcome === 'restored' ? 'ok' : 'warn';

function Result(props: { result: Finished; onClose: () => void }) {
  const failed = () => props.result.failed;
  return (
    <>
      <h3 class={failed() ? 'bad' : 'ok'}>
        <span class={failed() ? 'icon-pop' : 'icon-draw'}>{failed() ? <TriangleAlert size={20} /> : <CircleCheck size={20} />}</span>
        {props.result.title}{failed() ? ` with ${failed()} failure${failed() === 1 ? '' : 's'}` : ''}
      </h3>
      <div class="scroll">
        <table>
          <thead><tr><th>Project</th><th>Skill</th><th>Result</th><th>Detail</th></tr></thead>
          <tbody>
            <For each={props.result.lines}>{(line) => (
              <tr>
                <td class="path">{line.project}</td>
                <td>{line.skill}</td>
                <td><Chip tone={outcomeTone(line.outcome)}>{line.outcome}</Chip></td>
                <td class="muted pre">{line.detail}</td>
              </tr>
            )}</For>
          </tbody>
        </table>
      </div>
      <Show when={props.result.backup}>
        <p class="muted small">
          <HistoryIcon size={14} /> Backup run {props.result.backup}: <a data-link href="/history">History</a> can undo it.
        </p>
      </Show>
      <For each={props.result.warnings}>{(warning) => <p class="warn small">{warning}</p>}</For>
      <div class="actions">
        <button class="button button-primary" onClick={props.onClose}>Done</button>
        <button class="button" onClick={() => navigate('/')}>Back to the overview</button>
      </div>
    </>
  );
}
