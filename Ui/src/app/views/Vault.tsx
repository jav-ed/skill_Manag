import { FileText, Library, Search, TriangleAlert } from 'lucide-solid';
import { For, Show, createMemo, createResource, createSignal } from 'solid-js';
import { api } from '../api';
import { Chip, Empty, Loaded } from '../components';
import { appear } from '../motion';
import type { Place, PlaceState, VaultSkill } from '../types';

const placeTone = (state: PlaceState) => (state === 'current' ? 'ok' : state === 'problem' ? 'bad' : state === 'not_in_vault' ? 'muted' : 'warn');
const placeText: Record<PlaceState, string> = {
  current: 'up to date',
  outdated: 'outdated',
  missing: 'mandatory, not installed',
  not_in_vault: 'not in the vault',
  problem: 'cannot be compared',
};

function Where(props: { place: Place }) {
  return (
    <tr>
      <td class="path">{props.place.project}</td>
      <td><Chip tone={placeTone(props.place.state)}>{placeText[props.place.state]}</Chip></td>
      <td class="muted pre">{props.place.detail}</td>
    </tr>
  );
}

function Card(props: { skill: VaultSkill }) {
  const s = () => props.skill;
  return (
    <article class="card" ref={appear}>
      <h3>{s().name}<Show when={s().mandatory}><Chip tone="accent">mandatory</Chip></Show></h3>
      <p class="muted small">{s().group ? `group ${s().group}` : s().files.length ? 'top level of the vault' : 'not in the vault'}</p>
      <Show when={s().description}><p>{s().description}</p></Show>
      <Show when={s().header_problem}><p class="bad small"><TriangleAlert size={14} /> {s().header_problem}</p></Show>
      <Show when={s().profiles.length > 0}>
        <p class="small"><span class="muted">profiles</span> <For each={s().profiles}>{(p) => <Chip>{p}</Chip>}</For></p>
      </Show>
      <Show when={s().files.length > 0}>
        <h4>Files that are copied ({s().files.length})</h4>
        <ul class="files"><For each={s().files}>{(f) => <li><FileText size={13} />{f}</li>}</For></ul>
      </Show>
      <Show when={s().untracked.length > 0}>
        <p class="warn small"><TriangleAlert size={14} /> Not copied, git does not track: {s().untracked.join(', ')}</p>
      </Show>
      <h4>In the projects</h4>
      <Show when={s().projects.length > 0} fallback={<p class="muted small">Not installed in any project.</p>}>
        <div class="scroll"><table><tbody><For each={s().projects}>{(place) => <Where place={place} />}</For></tbody></table></div>
      </Show>
    </article>
  );
}

/** The vault's skills as a list with a card for the one that is picked. */
export default function Vault() {
  const [data, { refetch }] = createResource(api.vault);
  const [filter, setFilter] = createSignal('');
  const [picked, setPicked] = createSignal<string | null>(null);

  const all = createMemo(() => [...(data()?.skills ?? []), ...(data()?.foreign ?? [])]);
  const shown = createMemo(() => {
    const q = filter().trim().toLowerCase();
    return all().filter((s) => `${s.name} ${s.group}`.toLowerCase().includes(q));
  });
  const current = createMemo(() => shown().find((s) => s.name === picked()) ?? shown()[0]);

  return (
    <>
      <header class="view-head">
        <div>
          <h2>Skills</h2>
          <p class="lead">What the vault holds, and where each skill is installed.</p>
        </div>
      </header>
      <Loaded of={data} retry={refetch}>{(d) => (
        <Show when={all().length > 0} fallback={
          <Empty icon={<Library size={32} />} title="The vault has no skills yet">
            Make one with <code>skillmirror new NAME</code> or take one a project has with <code>skillmirror adopt NAME --from PROJECT</code>.
          </Empty>
        }>
          <div class="split">
            <div class="list-pane">
              <label class="search"><Search size={16} />
                <input type="search" placeholder="Filter skills" value={filter()} onInput={(e) => setFilter(e.currentTarget.value)} aria-label="Filter skills" />
              </label>
              <ul class="pick-list" role="listbox" aria-label="Skills">
                <For each={shown()}>{(s) => (
                  <li>
                    <button role="option" aria-selected={current()?.name === s.name} class={current()?.name === s.name ? 'pick on' : 'pick'} onClick={() => setPicked(s.name)}>
                      <strong>{s.name}</strong>
                      <span class="muted small">{s.group}</span>
                      <Show when={d.foreign.includes(s)}><Chip>not in the vault</Chip></Show>
                      <Show when={s.mandatory}><Chip tone="accent">mandatory</Chip></Show>
                    </button>
                  </li>
                )}</For>
              </ul>
              <Show when={shown().length === 0}><p class="muted small">No skill matches the filter.</p></Show>
            </div>
            <div class="card-pane"><Show when={current()}>{(s) => <Card skill={s()} />}</Show></div>
          </div>
        </Show>
      )}</Loaded>
    </>
  );
}
