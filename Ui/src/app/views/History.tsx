import { History as HistoryIcon, Undo2 } from 'lucide-solid';
import { For, Show, createResource, createSignal } from 'solid-js';
import { api } from '../api';
import { Cascade, Chip, Empty, Loaded, Spinner } from '../components';
import { UndoPanel } from '../panels';
import { notify, words } from '../toast';
import type { UndoPlan } from '../types';

/** The runs that kept a backup, and the undo of one of them. */
export default function History(props: { write: boolean }) {
  const [data, { refetch }] = createResource(api.history);
  const [plan, setPlan] = createSignal<UndoPlan | null>(null);
  const [asking, setAsking] = createSignal<string | null>(null);

  async function ask(run: string) {
    setAsking(run);
    try { setPlan(await api.undoPlan(run)); } catch (e) { notify(words(e)); } finally { setAsking(null); }
  }
  const close = () => { setPlan(null); void refetch(); };

  return (
    <>
      <header class="view-head">
        <div>
          <h2>History</h2>
          <p class="lead">Every run that replaced or removed a skill folder kept the old copy. Undoing a run brings it back; undoing is a run too, so it can be undone.</p>
        </div>
      </header>
      <Show when={!props.write}><p class="note">Start the server with <code>--allow-write</code> to undo a run from here.</p></Show>
      <Loaded of={data} retry={refetch}>{(d) => (
        <Show when={d.runs.length > 0} fallback={<Empty icon={<HistoryIcon size={32} />} title="No run has anything to undo">A sync, push, add, init or delete that replaces something will show up here.</Empty>}>
          <Cascade class="scroll table-wrap">
            <table>
              <thead><tr><th>Run</th><th>Command</th><th class="num">Skills</th><th class="num">Projects</th><th class="narrow" /></tr></thead>
              <tbody>
                <For each={d.runs}>{(run) => (
                  <tr>
                    <td class="path">{run.date}</td>
                    <td><Chip tone="accent">{run.command}</Chip></td>
                    <td class="num">{run.skills}</td>
                    <td class="num">{run.projects}</td>
                    <td class="right">
                      <Show when={run.error} fallback={
                        <Show when={props.write}>
                          <button class="button" onClick={() => ask(run.id)} disabled={asking() !== null}>
                            <Show when={asking() === run.id} fallback={<><Undo2 size={16} />Undo…</>}><Spinner /></Show>
                          </button>
                        </Show>
                      }><span class="bad small pre">{run.error}</span></Show>
                    </td>
                  </tr>
                )}</For>
              </tbody>
            </table>
          </Cascade>
        </Show>
      )}</Loaded>
      <Show when={plan()}>{(p) => <UndoPanel plan={p()} onClose={close} onApplied={() => void refetch()} />}</Show>
    </>
  );
}
