import type { BenchRow, CaseFile, CaseSummary } from './types';

async function get<T>(path: string): Promise<T> {
  const r = await fetch(path);
  if (!r.ok) throw new Error(`${r.status} ${path}`);
  return r.json() as Promise<T>;
}

export const listCases = () => get<CaseSummary[]>('/api/cases');
export const getCase = (id: string) => get<CaseFile>(`/api/cases/${encodeURIComponent(id)}`);
export const getBenchmark = () => get<BenchRow[]>('/api/benchmark');
export const caseFile = (id: string, rel: string) => `/api/cases/${encodeURIComponent(id)}/files/${rel}`;
