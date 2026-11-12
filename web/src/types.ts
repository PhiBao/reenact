export type VerdictKind = 'reproduced' | 'not_reproduced' | 'needs_info' | 'inconclusive';

export interface Verdict {
  kind: VerdictKind;
  headline: string;
  stuck_at: string | null;
  root_cause: string | null;
  suggested_fix: string | null;
  severity: string | null;
  confidence: number | null;
}

export interface Frame {
  ordinal: number;
  kind: string;
  label: string;
  status: string;
  file: string;
  annotated_file: string | null;
  rect: [number, number, number, number] | null;
  url: string | null;
}

export interface RunSummary {
  status: string;
  credits: number;
  duration_s: number;
  share_url: string | null;
  frames: Frame[];
  strip_file: string | null;
  signals: { console_errors: string[]; failed_requests: string[] };
  root_cause: string | null;
}

export interface RunRecord {
  kind: 'reenact' | 'confirm' | 'verify_fix';
  label: string;
  started_at: string;
  verdict: Verdict;
  run: RunSummary;
}

export interface FixAttempt {
  attempt: number;
  rationale: string;
  files: string[];
  applied: boolean;
  outcome: VerdictKind | null;
  note: string;
  regressions?: string[];
  suite_checked?: number;
}

export interface Plan {
  expected: string;
  reported_actual: string;
  steps: string[];
  preconditions: string[];
  missing: string[];
  confidence: number;
  summary: string;
}

export interface CaseFile {
  id: string;
  created_at: string;
  report: { title: string; body: string; source_url: string | null; repo: string | null; number: number | null };
  app_url: string;
  app_label: string | null;
  plan: Plan | null;
  test_md: string | null;
  runs: RunRecord[];
  fixes: FixAttempt[];
  pr_url: string | null;
  state: string;
}

export interface CaseSummary {
  id: string;
  title: string;
  verdict: string;
  state: string;
  headline: string;
  created_at: string;
  has_fix: boolean;
  strip: string;
  source_url: string | null;
}

export interface BenchSide {
  tag: string;
  verdict: string;
  headline: string;
  credits: number;
  duration_s: number;
  case_id: string;
  correct: boolean;
}

export interface BenchRow {
  id: string;
  kind: 'bug' | 'control';
  issue_url: string;
  title: string;
  buggy: BenchSide | null;
  fixed: BenchSide | null;
  error: string | null;
}

export const VERDICT_LABEL: Record<VerdictKind, string> = {
  reproduced: 'Reproduced',
  not_reproduced: 'Not reproduced',
  needs_info: 'Needs info',
  inconclusive: 'Inconclusive',
};
