import { For, Show, createResource } from 'solid-js';
import { api } from '../api';
import { Cascade, Chip, Loaded } from '../components';

const list = (items: string[]) => (items.length ? items.join(', ') : 'none');

/** What the server reads. It changes nothing; the command line and config.yaml do. */
export default function Settings() {
  const [data, { refetch }] = createResource(api.settings);
  return (
    <>
      <header class="view-head">
        <div>
          <h2>Settings</h2>
          <p class="lead">Change them with <code>skillmirror config</code> and <code>skillmirror mandatory</code>, or edit <code>config.yaml</code> in the vault.</p>
        </div>
      </header>
      <Loaded of={data} retry={refetch}>{(s) => (
        <Cascade class="settings">
          <div><dt>Vault</dt><dd><Show when={s.vault} fallback={<Chip tone="warn">not set</Chip>}>{(v) => <><span class="path">{v().path}</span> <Chip>{v().source}</Chip></>}</Show></dd></div>
          <div><dt>Scan root</dt><dd><Show when={s.root} fallback={<Chip tone="warn">not set</Chip>}>{(r) => <><span class="path">{r().path}</span> <Chip>{r().source}</Chip></>}</Show></dd></div>
          <div><dt>Mandatory skills</dt><dd>{list(s.mandatory)}</dd></div>
          <div><dt>Targets</dt><dd>{list(s.targets)}</dd></div>
          <div><dt>Excluded names</dt><dd>{list(s.exclude_dirs)}</dd></div>
          <div><dt>Excluded paths</dt><dd class="path">{list(s.exclude_paths)}</dd></div>
          <div><dt>Profiles</dt><dd><Show when={s.profiles.length > 0} fallback="none"><For each={s.profiles}>{(p) => <Chip title={p.description ?? ''}>{p.name}</Chip>}</For></Show></dd></div>
          <div><dt>Changes from this page</dt><dd><Chip tone={s.allow_write ? 'warn' : 'muted'}>{s.allow_write ? 'allowed (--allow-write)' : 'not allowed'}</Chip></dd></div>
        </Cascade>
      )}</Loaded>
    </>
  );
}
