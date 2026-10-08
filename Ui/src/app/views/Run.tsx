import { ListChecks, ListX, Search, Wand2 } from 'lucide-solid';
import { For, Show, createMemo, createResource, createSignal } from 'solid-js';
import { api } from '../api';
import { Cascade, Chip, Empty, Loaded, Spinner } from '../components';
import { PlanPanel } from '../panels';
import { query } from '../router';
import { notify, words } from '../toast';
import type { Plan, SkillRow } from '../types';

const copy = {
  sync: {
    title: 'Sync',
    lead: 'Refreshes the skills each project already has from the vault. It never adds a skill to a project that does not have it.',
  },
  push: {
    title: 'Push',
    lead: 'Installs the mandatory skills in every project that has a skills folder, and refreshes the ones that are already there.',
  },
} as const;

const changes = (row: SkillRow) => row.outdated + row.missing;

/** The skills to pick from, and the plan of the pick. Sync and push are the same page with other rows. */
export default function Run(props: { kind: 'sync' | 'push'; write: boolean }) {
  const [data, { refetch }] = createResource(() => props.kind, api.skills);
  const [filter, setFilter] = createSignal('');
  const [picked, setPicked] = createSignal<Set<string> | null>(null);
  const [project, setProject] = createSignal(query().get('project') ?? '');
  const [plan, setPlan] = createSignal<Plan | null>(null);
  const [planning, setPlanning] = createSignal(false);

  // Until the person ticks something themselves, the skills that would change are ticked.
  const chosen = createMemo(() => picked() ?? new Set((data()?.rows ?? []).filter((r) => changes(r) > 0).map((r) => r.name)));
  const visible = createMemo(() => (data()?.rows ?? []).filter((r) => `${r.name} ${r.group}`.toLowerCase().includes(filter().trim().toLowerCase())));

  const toggle = (name: string) => {
    const next = new Set(chosen());
    if (!next.delete(name)) next.add(name);
    setPicked(next);
  };
  const only = (keep: (r: SkillRow) => boolean) => setPicked(new Set(visible().filter(keep).map((r) => r.name)));

  async function makePlan() {
    setPlanning(true);
    try {
      setPlan(await api.plan(props.kind, [...chosen()], project() || null));
    } catch (error) {
      notify(words(error));
    } finally {
      setPlanning(false);
    }
  }
  const close = () => { setPlan(null); void refetch(); };

  return (
    <>
      <header class="view-head">
        <div>
          <h2>{copy[props.kind].title}</h2>
          <p class="lead">{copy[props.kind].lead}</p>
        </div>
      </header>
      <Show when={!props.write}>
        <p class="note">This server only looks. Start it with <code>skillmirror web --allow-write</code> to run a {props.kind} from here.</p>
      </Show>
      <Loaded of={data} retry={refetch}>{(d) => (
        <>
          <div class="toolbar">
            <label class="search"><Search size={16} />
              <input type="search" placeholder="Filter skills" value={filter()} onInput={(e) => setFilter(e.currentTarget.value)} aria-label="Filter skills" />
            </label>
            <Show when={props.write}>
              <select value={project()} onChange={(e) => setProject(e.currentTarget.value)} aria-label="Project">
                <option value="">All projects</option>
                <For each={d.projects}>{(p) => <option value={p.path}>{p.label}</option>}</For>
              </select>
              <button class="button" onClick={() => only((r) => changes(r) > 0)}><Wand2 size={16} />Select changed</button>
              <button class="button" onClick={() => only(() => true)}><ListChecks size={16} />All</button>
              <button class="button" onClick={() => setPicked(new Set())}><ListX size={16} />None</button>
              <button class="button button-primary" onClick={makePlan} disabled={planning() || chosen().size === 0}>
                <Show when={planning()} fallback="Plan…"><Spinner label="Planning…" /></Show>
              </button>
            </Show>
          </div>
          <Show when={d.rows.length > 0} fallback={<Empty icon={<ListChecks size={32} />} title={`No skill to ${props.kind}`}>
            {props.kind === 'push' ? 'The vault config lists no mandatory skill. Add one with `skillmirror mandatory add`.' : 'No project has a skill that the vault also has.'}
          </Empty>}>
            <Cascade class="scroll table-wrap">
              <table class="skills">
                <thead><tr>
                  <Show when={props.write}><th class="narrow" /></Show>
                  <th>Skill</th><th>Group</th><th class="num">Installed in</th><th>State</th>
                </tr></thead>
                <tbody>
                  <For each={visible()}>{(row) => (
                    <tr>
                      <Show when={props.write}>
                        <td class="narrow"><input type="checkbox" checked={chosen().has(row.name)} onChange={() => toggle(row.name)} aria-label={row.name} /></td>
                      </Show>
                      <td><strong>{row.name}</strong><Show when={row.mandatory}><Chip tone="accent">mandatory</Chip></Show></td>
                      <td class="muted">{row.group}</td>
                      <td class="num">{row.current + row.outdated}</td>
                      <td class="chips">
                        <Show when={row.problems > 0}><Chip tone="bad">{row.problems} cannot be compared</Chip></Show>
                        <Show when={row.outdated > 0}><Chip tone="warn">{row.outdated} outdated</Chip></Show>
                        <Show when={row.missing > 0}><Chip tone="warn">{row.missing} missing</Chip></Show>
                        <Show when={changes(row) === 0 && row.problems === 0}><Chip tone="ok">up to date</Chip></Show>
                      </td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
            </Cascade>
          </Show>
        </>
      )}</Loaded>
      <Show when={plan()}>{(p) => <PlanPanel plan={p()} onClose={close} onApplied={() => void refetch()} />}</Show>
    </>
  );
}
