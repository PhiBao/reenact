import { useEffect, useState } from 'react';
import { getBenchmark } from '../api';
import type { BenchRow, BenchSet, BenchSide } from '../types';
import { Stamp, kindFromLabel } from '../components/Stamp';

function Side({ s }: { s: BenchSide | null }) {
  if (!s) return <span className="muted">harness error</span>;
  return (
    <span className={`side ${s.correct ? 'side-ok' : 'side-miss'}`}>
      <Stamp kind={kindFromLabel(s.verdict)} />
      <small>
        {s.tag}
        {s.first_run && s.first_run !== s.verdict ? ` · first run: ${s.first_run}` : ''}
      </small>
    </span>
  );
}

function tally(rows: BenchRow[]) {
  const scored = rows.filter((r) => r.buggy && r.fixed);
  const bugs = scored.filter((r) => r.kind === 'bug');
  const controls = scored.filter((r) => r.kind === 'control');
  return {
    bothOk: scored.filter((r) => r.buggy!.correct && r.fixed!.correct).length,
    scored: scored.length,
    bugs: bugs.length,
    found: bugs.filter((r) => r.buggy!.verdict === 'Reproduced').length,
    foundFirst: bugs.filter((r) => (r.buggy!.first_run ?? r.buggy!.verdict) === 'Reproduced').length,
    falseOnFixed: bugs.filter((r) => r.fixed!.verdict === 'Reproduced').length,
    controlsFlagged: controls.reduce(
      (n, r) => n + (['buggy', 'fixed'] as const).filter((k) => r[k]!.verdict === 'Reproduced').length,
      0,
    ),
    controlRuns: controls.length * 2,
    errors: rows.length - scored.length,
  };
}

export function Benchmark() {
  const [sets, setSets] = useState<BenchSet[] | null>(null);
  const [active, setActive] = useState(0);
  const [err, setErr] = useState(false);
  useEffect(() => {
    getBenchmark()
      .then(setSets)
      .catch(() => setErr(true));
  }, []);

  if (err) return <p className="empty">Benchmark results are not published yet.</p>;
  if (!sets) return <p className="empty">Loading…</p>;
  const set = sets[active] ?? sets[0];
  const t = tally(set.rows);

  return (
    <section className="bench">
      <p className="eyebrow">Historical benchmark</p>
      <h1 className="case-title">Real issues, real releases.</h1>
      <p className="lede">
        Each row is a closed bug report from Memos, an open-source note app. Reenact compiles the report once, then runs
        the same test against the release that had the bug and the release that fixed it. A correct result is
        “Reproduced” then “Not reproduced”. Controls are reports that are not bugs, so both would ideally be “Not
        reproduced”.
      </p>

      {sets.length > 1 && (
        <div className="tabs" role="tablist" aria-label="Benchmark sets">
          {sets.map((s, i) => (
            <button key={s.id} role="tab" aria-selected={i === active} className={i === active ? 'on' : ''} onClick={() => setActive(i)}>
              {s.title}
            </button>
          ))}
        </div>
      )}
      {set.note && <p className="muted bench-note">{set.note}</p>}

      <dl className="tally">
        <div>
          <dt>Bugs reproduced on the buggy release</dt>
          <dd>
            {t.found} / {t.bugs}
          </dd>
        </div>
        <div>
          <dt>…by the very first run, without a retry</dt>
          <dd>
            {t.foundFirst} / {t.bugs}
          </dd>
        </div>
        <div>
          <dt>Bugs wrongly reproduced on the fixed release</dt>
          <dd>
            {t.falseOnFixed} / {t.bugs}
          </dd>
        </div>
        <div>
          <dt>Non-bug controls flagged as reproduced</dt>
          <dd>
            {t.controlsFlagged} / {t.controlRuns}
          </dd>
        </div>
      </dl>
      <p className="muted">
        {t.bothOk} of {t.scored} cases judged exactly right on both releases.
        {t.errors > 0 ? ` ${t.errors} case(s) not scored because the test harness failed before Reenact ran.` : ''}
      </p>

      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Issue</th>
              <th>Kind</th>
              <th>Buggy release</th>
              <th>Fixed release</th>
            </tr>
          </thead>
          <tbody>
            {set.rows.map((r) => (
              <tr key={r.id}>
                <td>
                  <a href={r.issue_url}>{r.title}</a>
                </td>
                <td>{r.kind}</td>
                <td>
                  <Side s={r.buggy} />
                </td>
                <td>
                  <Side s={r.fixed} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="muted">
        Misses are shown, not hidden. A control flagged “Reproduced” means the page really behaves as the report
        describes; whether that is a bug is the maintainer’s call, which Reenact cannot know.
      </p>
    </section>
  );
}
