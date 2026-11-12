import { useEffect, useState } from 'react';
import { getBenchmark } from '../api';
import type { BenchRow, BenchSide } from '../types';
import { Stamp, kindFromLabel } from '../components/Stamp';

function Side({ s }: { s: BenchSide | null }) {
  if (!s) return <span className="muted">error</span>;
  return (
    <span className={`side ${s.correct ? 'side-ok' : 'side-miss'}`}>
      <Stamp kind={kindFromLabel(s.verdict)} />
      <small>{s.tag}</small>
    </span>
  );
}

export function Benchmark() {
  const [rows, setRows] = useState<BenchRow[] | null>(null);
  const [err, setErr] = useState(false);
  useEffect(() => {
    getBenchmark().then(setRows).catch(() => setErr(true));
  }, []);

  if (err) return <p className="empty">Benchmark results are not published yet.</p>;
  if (!rows) return <p className="empty">Loading…</p>;

  const scored = rows.filter((r) => r.buggy && r.fixed);
  const ok = scored.filter((r) => r.buggy!.correct && r.fixed!.correct);
  const bugs = scored.filter((r) => r.kind === 'bug');
  const caught = bugs.filter((r) => r.buggy!.correct);
  const cleared = bugs.filter((r) => r.fixed!.correct);
  const controls = scored.filter((r) => r.kind === 'control');
  const falsePos = controls.filter((r) => !r.buggy!.correct || !r.fixed!.correct);

  return (
    <section className="bench">
      <p className="eyebrow">Historical benchmark</p>
      <h1 className="case-title">Real issues, real releases.</h1>
      <p className="lede">
        Each row is a closed bug report from Memos, an open-source note app. Reenact compiles the report once, then
        runs the same test against the release that had the bug and the release that fixed it. A correct result is
        “Reproduced” then “Not reproduced”. Controls are reports that are not bugs, so both must be “Not reproduced”.
      </p>
      <dl className="tally">
        <div><dt>Judged correctly on both releases</dt><dd>{ok.length} / {scored.length}</dd></div>
        <div><dt>Bugs reproduced on the buggy release</dt><dd>{caught.length} / {bugs.length}</dd></div>
        <div><dt>Bugs cleared on the fixed release</dt><dd>{cleared.length} / {bugs.length}</dd></div>
        <div><dt>Controls wrongly flagged</dt><dd>{falsePos.length} / {controls.length}</dd></div>
      </dl>
      <div className="table-wrap">
        <table>
          <thead>
            <tr><th>Issue</th><th>Kind</th><th>Buggy release</th><th>Fixed release</th></tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td><a href={r.issue_url}>{r.title}</a></td>
                <td>{r.kind}</td>
                <td><Side s={r.buggy} /></td>
                <td><Side s={r.fixed} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="muted">Misses are shown, not hidden. Evidence for every run is in <code>benchmark/runs/</code>.</p>
    </section>
  );
}
