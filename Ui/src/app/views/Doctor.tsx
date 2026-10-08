import { CircleCheck, CircleX, Info, TriangleAlert } from 'lucide-solid';
import { For, Show, createResource } from 'solid-js';
import { api } from '../api';
import { Cascade, Empty, Loaded } from '../components';

const icon = { note: <Info size={18} />, warning: <TriangleAlert size={18} />, error: <CircleX size={18} /> } as const;
const tone = { note: 'muted', warning: 'warn', error: 'bad' } as const;

/** What `skillmirror doctor` finds, read-only. */
export default function Doctor() {
  const [data, { refetch }] = createResource(api.doctor);
  return (
    <>
      <header class="view-head">
        <div>
          <h2>Doctor</h2>
          <p class="lead">The machine, the configuration, the vault and every SKILL.md header. Nothing is written.</p>
        </div>
      </header>
      <Loaded of={data} retry={refetch}>{(d) => (
        <Show when={d.findings.length > 0} fallback={<Empty icon={<CircleCheck size={32} />} title="Everything checked out" />}>
          <Cascade class="findings">
            <For each={d.findings}>{(f) => (
              <div class={`finding finding-${tone[f.severity]}`}>
                <span class="finding-icon">{icon[f.severity]}</span>
                <div>
                  <p><strong>{f.check}</strong><Show when={f.subject}> <span class="path muted">{f.subject}</span></Show></p>
                  <p class="pre">{f.message}</p>
                  <Show when={f.hint}><p class="muted small">hint: {f.hint}</p></Show>
                </div>
              </div>
            )}</For>
          </Cascade>
        </Show>
      )}</Loaded>
    </>
  );
}
