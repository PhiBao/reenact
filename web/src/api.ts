import type { BenchRow, BenchSet, CaseFile, CaseSummary } from './types';

async function get<T>(path: string): Promise<T> {
  const r = await fetch(path);
  if (!r.ok) throw new Error(`${r.status} ${path}`);
  return r.json() as Promise<T>;
}

export const listCases = () => get<CaseSummary[]>('/api/cases');
export const getCase = (id: string) => get<CaseFile>(`/api/cases/${encodeURIComponent(id)}`);
export async function getBenchmark(): Promise<BenchSet[]> {
  const raw = await get<BenchRow[] | { sets: BenchSet[] }>('/api/benchmark');
  return Array.isArray(raw) ? [{ id: 'main', title: 'Benchmark', note: '', rows: raw }] : raw.sets;
}
export const caseFile = (id: string, rel: string) => `/api/cases/${encodeURIComponent(id)}/files/${rel}`;
