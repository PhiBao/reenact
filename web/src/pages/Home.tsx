import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { getBenchmark, listCases, caseFile } from '../api';
import type { BenchSet, CaseSummary } from '../types';
import { Stamp, kindFromLabel } from '../components/Stamp';
import { Compare } from '../components/Compare';

const STEPS = [
  { n: '01', t: 'The report', d: 'A GitHub issue, exactly as the reporter wrote it.' },
  { n: '02', t: 'The test', d: 'Rendered into a plain-English Kane test. The model never writes the test file.' },
  { n: '03', t: 'The reenactment', d: 'Kane drives a real browser. Its triage decides: reproduced, not reproduced, or needs info.' },
  { n: '04', t: 'The fix', d: 'An agent edits the code. The same test has to pass, or it tries again.' },
];

function headline(sets: BenchSet[]) {
  const rows = sets[0].rows.filter((r) => r.kind === 'bug' && r.buggy && r.fixed);
  return {
    title: sets[0].title,
    bugs: rows.length,
    found: rows.filter((r) => r.buggy!.verdict === 'Reproduced').length,
    falseOnFixed: rows.filter((r) => r.fixed!.verdict === 'Reproduced').length,
  };
}

export function Home() {
  const [cases, setCases] = useState<CaseSummary[] | null>(null);
  const [bench, setBench] = useState<BenchSet[] | null>(null);

  useEffect(() => {
    listCases().then((c) => setCases([...c].sort((a, b) => Number(b.has_fix) - Number(a.has_fix)))).catch(() => setCases([]));
    getBenchmark().then(setBench).catch(() => setBench(null));
  }, []);

  const featured =
    cases?.find((c) => c.has_fix && (c.source_url ?? '').includes('usememos/memos')) ?? cases?.find((c) => c.has_fix) ?? cases?.[0];
  const totals = bench && bench.length ? headline(bench) : null;

  return (
    <>
      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">Bug reports, verified</p>
          <h1>
            Every bug report,
            <br />
            <em>reenacted.</em>
          </h1>
          <p className="lede">
            Reenact reads the issue, makes a real browser do what the reporter did, and tells you whether the bug is
            real. Then it proves the fix with the same test.
          </p>
          <p className="lede-sub">Built on Kane CLI. Works on any web app. No test code, no selectors.</p>
        </div>
        <div className="hero-proof">
          {featured ? (
            <Link to={`/c/${featured.id}`} className="proof-link" aria-label={`Open case ${featured.title}`}>
              {featured.has_fix ? (
                <Compare before={caseFile(featured.id, 'before.png')} after={caseFile(featured.id, 'after.png')} />
              ) : (
                <img src={caseFile(featured.id, featured.strip)} alt={`Kane's run for: ${featured.title}`} />
              )}
              <span className="proof-cap">
                <Stamp kind={kindFromLabel(featured.verdict)} />
                <span>{featured.title}</span>
              </span>
            </Link>
          ) : (
            <div className="proof-empty">The first case file will appear here.</div>
          )}
        </div>
      </section>

      <section className="steps" aria-label="How it works">
        <ol>
          {STEPS.map((s, i) => (
            <li key={s.n} style={{ animationDelay: `${i * 90}ms` }}>
              <span className="step-n">{s.n}</span>
              <h3>{s.t}</h3>
              <p>{s.d}</p>
            </li>
          ))}
        </ol>
      </section>

      {totals && totals.bugs > 0 && (
        <section className="bench-line">
          <p>
            <strong>
              {totals.found} of {totals.bugs}
            </strong>{' '}
            real Memos bugs reproduced on unseen reports; {totals.falseOnFixed} wrongly reproduced on the fixed release. The misses are published.
          </p>
          <Link to="/benchmark">See every case →</Link>
        </section>
      )}

      <section aria-label="Case files" className="feed">
        <h2>Case files</h2>
        {cases === null && <p className="empty">Loading…</p>}
        {cases?.length === 0 && <p className="empty">No cases yet. File an issue on a repo that runs Reenact.</p>}
        <ul>
          {cases?.map((c) => (
            <li key={c.id}>
              <Link to={`/c/${c.id}`}>
                <span className="feed-id">{c.id}</span>
                <span className="feed-title">
                  {c.title}
                  {c.source_url && <small className="feed-src">{c.source_url.replace('https://github.com/', '').replace(/\/issues\/\d+$/, '')}</small>}
                </span>
                <span className="feed-verdict">
                  <Stamp kind={kindFromLabel(c.verdict)} />
                  {c.has_fix && <span className="fix-chip">fix verified</span>}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}
