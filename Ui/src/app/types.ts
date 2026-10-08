// The shapes of the JSON the server sends. They mirror `Crates/Web/src/server/api/` and are pinned by the
// tests in `Crates/Web/src/server/tests/read.rs`; see Project_Manag/Docs/Architecture/web_Api.md.

export interface Session { allow_write: boolean; version: string }

export interface Outdated { skill: string; added: number; changed: number; removed: number }
export interface Problem { name: string; message: string; hint: string | null }
export type ProjectState = 'in_sync' | 'drift' | 'problem';
export interface Project {
  path: string;
  label: string;
  state: ProjectState;
  current: number;
  outdated: Outdated[];
  missing: string[];
  missing_links: string[];
  not_in_vault: string[];
  problems: Problem[];
  link_problems: Problem[];
}
export interface Overview {
  at: string;
  vault: string;
  root: string;
  projects: Project[];
  in_sync: number;
  drift: number;
  problems: number;
  issues: { path: string; message: string }[];
}

export interface SkillRow {
  name: string;
  group: string;
  mandatory: boolean;
  current: number;
  outdated: number;
  missing: number;
  problems: number;
}
export interface Skills { rows: SkillRow[]; projects: { path: string; label: string }[] }

export type PlaceState = 'current' | 'outdated' | 'missing' | 'not_in_vault' | 'problem';
export interface Place { project: string; state: PlaceState; detail: string | null }
export interface VaultSkill {
  name: string;
  group: string;
  mandatory: boolean;
  description: string | null;
  header_problem: string | null;
  files: string[];
  untracked: string[];
  profiles: string[];
  projects: Place[];
}
export interface Vault { skills: VaultSkill[]; foreign: VaultSkill[] }

export interface Run { id: string; date: string; command: string; skills: number; projects: number; error: string | null }
export interface History { runs: Run[] }

export interface Finding { severity: 'note' | 'warning' | 'error'; check: string; subject: string | null; message: string; hint: string | null }
export interface Doctor { findings: Finding[] }

export interface Located { path: string; source: string }
export interface Settings {
  vault: Located | null;
  root: Located | null;
  mandatory: string[];
  targets: string[];
  exclude_dirs: string[];
  exclude_paths: string[];
  profiles: { name: string; description: string | null }[];
  allow_write: boolean;
}

export interface DiffFile { path: string; kind: 'added' | 'changed' | 'mode' | 'removed'; added: number; removed: number; text: string; note: string | null }
export interface PlanRow { project: string; skill: string; action: 'create' | 'update' | 'unchanged' | 'failed'; detail: string; files: DiffFile[] }
export interface Plan {
  plan: string;
  kind: string;
  rows: PlanRow[];
  create: number;
  update: number;
  unchanged: number;
  failed: number;
  projects: number;
}

export interface UndoLine { project: string; skill: string; was: string; step: 'restore' | 'remove' | 'gone' | 'failed'; message: string | null }
export interface UndoPlan { plan: string; run: string; date: string; command: string; lines: UndoLine[]; actionable: number; failed: number }

export interface ResultLine { project: string; skill: string; outcome: string; detail: string | null }
export type JobView =
  | { state: 'running'; done: number; total: number }
  | { state: 'done'; title: string; lines: ResultLine[]; failed: number; backup: string | null; warnings: string[] }
  | { state: 'failed'; message: string };
export type Finished = Extract<JobView, { state: 'done' }>;
