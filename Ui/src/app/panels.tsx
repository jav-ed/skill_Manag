import { For, Show } from 'solid-js';
import { Chip, DiffView } from './components';
import { Flow } from './flow';
import type { Plan, UndoPlan } from './types';

const actionTone = (action: string) =>
  action === 'failed' ? 'bad' : action === 'create' ? 'ok' : action === 'update' ? 'warn' : 'muted';

/** What a sync or push would write, with the lines of every file, and the button that writes it. */
export function PlanPanel(props: { plan: Plan; onClose: () => void; onApplied?: () => void }) {
  const changes = () => props.plan.create + props.plan.update;
  return (
    <Flow
      planId={props.plan.plan}
      apply="Apply this plan"
      disabled={changes() === 0}
      onClose={props.onClose}
      onApplied={props.onApplied}
      review={(
        <>
          <h3>Plan: {props.plan.kind}</h3>
          <p class="chips">
            <Chip tone="ok">{props.plan.create} to create</Chip>
            <Chip tone="warn">{props.plan.update} to update</Chip>
            <Chip>{props.plan.unchanged} unchanged</Chip>
            <Show when={props.plan.failed > 0}><Chip tone="bad">{props.plan.failed} cannot be applied</Chip></Show>
            <Chip>{props.plan.projects} project{props.plan.projects === 1 ? '' : 's'}</Chip>
          </p>
          <Show when={changes() === 0}><p class="muted">Nothing would be written.</p></Show>
          <div class="scroll">
            <table>
              <thead><tr><th>Project</th><th>Skill</th><th>Action</th><th>Changes</th></tr></thead>
              <tbody>
                <For each={props.plan.rows}>{(row) => (
                  <tr>
                    <td class="path">{row.project}</td>
                    <td>{row.skill}</td>
                    <td><Chip tone={actionTone(row.action)}>{row.action}</Chip></td>
                    <td>
                      <span class="muted pre">{row.detail}</span>
                      <For each={row.files}>{(file) => <DiffView file={file} />}</For>
                    </td>
                  </tr>
                )}</For>
              </tbody>
            </table>
          </div>
        </>
      )}
    />
  );
}

const stepTone = (step: string) => (step === 'failed' ? 'bad' : step === 'gone' ? 'muted' : step === 'remove' ? 'warn' : 'ok');

/** What undoing a run would do, and the button that does it. */
export function UndoPanel(props: { plan: UndoPlan; onClose: () => void; onApplied?: () => void }) {
  return (
    <Flow
      planId={props.plan.plan}
      apply="Undo this run"
      danger
      disabled={props.plan.actionable === 0}
      onClose={props.onClose}
      onApplied={props.onApplied}
      review={(
        <>
          <h3>Undo {props.plan.command} of {props.plan.date}</h3>
          <p class="chips">
            <Chip tone="ok">{props.plan.actionable} can be put right</Chip>
            <Show when={props.plan.failed > 0}><Chip tone="bad">{props.plan.failed} cannot</Chip></Show>
          </p>
          <div class="scroll">
            <table>
              <thead><tr><th>Project</th><th>Skill</th><th>The run</th><th>Undo</th></tr></thead>
              <tbody>
                <For each={props.plan.lines}>{(line) => (
                  <tr>
                    <td class="path">{line.project}</td>
                    <td>{line.skill}</td>
                    <td class="muted">{line.was}</td>
                    <td>
                      <Chip tone={stepTone(line.step)}>{line.step}</Chip>
                      <Show when={line.message}><span class="muted pre"> {line.message}</span></Show>
                    </td>
                  </tr>
                )}</For>
              </tbody>
            </table>
          </div>
        </>
      )}
    />
  );
}
