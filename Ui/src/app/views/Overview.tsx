import { ChevronDown, CircleCheck, CircleDot, Folder, FolderCheck, RefreshCw, TriangleAlert } from 'lucide-solid';
import { For, Show, createResource, createSignal } from 'solid-js';
import { api } from '../api';
import { Cascade, Chip, Empty, Loaded, Stat } from '../components';
import { appear } from '../motion';
import { navigate } from '../router';
import { notify, words } from '../toast';
import type { Project } from '../types';

function ProjectRow(props: { project: Project }) {
  const [open, setOpen] = createSignal(false);
  const p = () => props.project;
  const icon = () => (p().state === 'in_sync' ? <CircleCheck size={18} /> : p().state === 'drift' ? <CircleDot size={18} /> : <TriangleAlert size={18} />);
  const needsWork = () => p().outdated.length + p().missing.length + p().missing_links.length > 0;
  return (
    <li class={`project project-${p().state}`}>
      <button class="project-head" onClick={() => setOpen(!open())} aria-expanded={open()}>
        <span class={`state-icon state-${p().state}`}>{icon()}</span>
        <span class="path grow">{p().label}</span>
        <span class="chips">
          <Show when={p().outdated.length > 0}><Chip tone="warn">{p().outdated.length} outdated</Chip></Show>
          <Show when={p().missing.length > 0}><Chip tone="warn">{p().missing.length} missing</Chip></Show>
          <Show when={p().problems.length + p().link_problems.length > 0}><Chip tone="bad">{p().problems.length + p().link_problems.length} problem</Chip></Show>
          <Show when={p().state === 'in_sync'}><Chip tone="ok">in sync</Chip></Show>
        </span>
        <ChevronDown size={18} class={open() ? 'chevron open' : 'chevron'} />
      </button>
      <Show when={open()}>
        <div class="project-body" ref={appear}>
          <p class="muted small path">{p().path}</p>
          <ul class="detail-list">
            <For each={p().outdated}>{(o) => (
              <li><Chip tone="warn">outdated</Chip> <strong>{o.skill}</strong> <span class="muted">
                {[o.added && `${o.added} added`, o.changed && `${o.changed} changed`, o.removed && `${o.removed} removed`].filter(Boolean).join(', ')}
              </span></li>
            )}</For>
            <For each={p().missing}>{(name) => <li><Chip tone="warn">missing</Chip> <strong>{name}</strong> <span class="muted">mandatory, not installed</span></li>}</For>
            <For each={p().missing_links}>{(name) => <li><Chip tone="warn">link</Chip> <strong>{name}</strong> <span class="muted">not linked to .agents/skills yet</span></li>}</For>
            <For each={p().problems}>{(x) => (
              <li><Chip tone="bad">problem</Chip> <strong>{x.name}</strong> <span class="pre">{x.message}</span>
                <Show when={x.hint}><div class="muted small">hint: {x.hint}</div></Show></li>
            )}</For>
            <For each={p().link_problems}>{(x) => (
              <li><Chip tone="bad">link</Chip> <strong>{x.name}</strong> <span class="pre">{x.message}</span>
                <Show when={x.hint}><div class="muted small">hint: {x.hint}</div></Show></li>
            )}</For>
            <For each={p().not_in_vault}>{(name) => <li><Chip>not in the vault</Chip> <strong>{name}</strong> <span class="muted">sync leaves it alone</span></li>}</For>
            <Show when={p().current > 0}><li class="muted">{p().current} skill{p().current === 1 ? '' : 's'} up to date</li></Show>
          </ul>
          <Show when={needsWork()}>
            <button class="button button-primary" onClick={() => navigate(`/sync?project=${encodeURIComponent(p().path)}`)}>
              <RefreshCw size={16} />Sync this project…
            </button>
          </Show>
        </div>
      </Show>
    </li>
  );
}

export default function Overview() {
  const [data, { refetch }] = createResource(api.overview);
  const [scanning, setScanning] = createSignal(false);
  async function rescan() {
    setScanning(true);
    try { await api.rescan(); await refetch(); } catch (e) { notify(words(e)); } finally { setScanning(false); }
  }
  return (
    <>
      <header class="view-head">
        <div>
          <h2>Overview</h2>
          <p class="lead">How every project stands against the vault.</p>
        </div>
        <button class="button" onClick={rescan} disabled={scanning()}>
          <RefreshCw size={16} class={scanning() ? 'spin' : 'turn-on-hover'} />Rescan
        </button>
      </header>
      <Loaded of={data} retry={refetch}>{(d) => (
        <>
          <Cascade class="stats">
            <Stat label="projects" value={d.projects.length} icon={<Folder size={22} />} />
            <Stat label="in sync" value={d.in_sync} tone="ok" icon={<FolderCheck size={22} />} />
            <Stat label="differ" value={d.drift} tone="warn" icon={<CircleDot size={22} />} />
            <Stat label="with a problem" value={d.problems} tone={d.problems ? 'bad' : undefined} icon={<TriangleAlert size={22} />} />
          </Cascade>
          <p class="muted small meta">Vault <span class="path">{d.vault}</span> · root <span class="path">{d.root}</span> · looked at {d.at}</p>
          <Show when={d.projects.length > 0} fallback={
            <Empty icon={<Folder size={32} />} title="No project has a skills folder">
              Nothing under <span class="path">{d.root}</span> has a <code>.agents/skills</code> folder yet.
            </Empty>
          }>
            <Cascade class="project-list"><ul>
              <For each={d.projects}>{(project) => <ProjectRow project={project} />}</For>
            </ul></Cascade>
          </Show>
          <Show when={d.issues.length > 0}>
            <section class="panel">
              <h3><TriangleAlert size={18} /> The scan could not read</h3>
              <ul class="detail-list"><For each={d.issues}>{(i) => <li><span class="path">{i.path}</span> <span class="muted">{i.message}</span></li>}</For></ul>
            </section>
          </Show>
        </>
      )}</Loaded>
    </>
  );
}
