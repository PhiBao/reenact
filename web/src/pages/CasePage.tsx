import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { caseFile, getCase } from '../api';
import { VERDICT_LABEL, type CaseFile, type RunRecord } from '../types';
import { Stamp } from '../components/Stamp';
import { Compare } from '../components/Compare';

/** Issue templates are mostly boilerplate. Show the parts that describe the bug. */
export function compactReport(body: string): string {
  const cleaned = body.replace(/<img[^>]*>/gi, '').replace(/!\[[^\]]*\]\([^)]*\)/g, '').replace(/\r/g, '');
  const parts = cleaned.split(/^###\s+/m);
  if (parts.length < 3) return cleaned.trim();
  const skip = /(checklist|check before|pre-submission|where did you|version|deployment|client env|additional context|screenshots?)/i;
  const kept = parts
    .slice(1)
    .map((p) => {
      const nl = p.indexOf('\n');
      return { head: p.slice(0, nl < 0 ? p.length : nl).trim(), text: nl < 0 ? '' : p.slice(nl + 1).trim() };
    })
    .filter((x) => !skip.test(x.head) && x.text && x.text !== '_No response_');
  if (!kept.length) return cleaned.trim();
  return kept.map((x) => (kept.length > 1 ? `${x.head}\n${x.text}` : x.text)).join('\n\n');
}

function Steps({ run }: { run: RunRecord }) {
  const frames = run.run.frames.filter((f) => f.kind !== 'navigate' && f.kind !== 'wait' && f.label);
  if (!frames.length) return null;
  return (
    <ol className="doing">
      {frames.map((f, i) => (
        <li key={f.ordinal}>
          <span className="n">{String(i + 1).padStart(2, '0')}</span>
          <span className="what">{f.label}</span>
          <span className={`res res-${f.status}`}>{f.status}</span>
        </li>
      ))}
    </ol>
  );
}

export function CasePage() {
  const { id = '' } = useParams();
  const [c, setC] = useState<CaseFile | null>(null);
  const [err, setErr] = useState(false);

  useEffect(() => {
    setC(null);
    setErr(false);
    getCase(id).then(setC).catch(() => setErr(true));
  }, [id]);

  useEffect(() => {
    if (c) document.title = `Reenact · ${c.report.title}`;
  }, [c]);

  if (err) return <p className="empty">No case file named {id}.</p>;
  if (!c) return <p className="empty">Loading…</p>;

  const repro = [...c.runs].reverse().find((r) => r.kind === 'reenact');
  const fix = [...c.runs].reverse().find((r) => r.kind === 'verify_fix');
  if (!repro) return <p className="empty">This case has not been run yet.</p>;
  const v = repro.verdict;
  const hasStrip = repro.run.frames.length > 0;
  const fixed = fix?.verdict.kind === 'not_reproduced';
  const confirm = [...c.runs].reverse().find((r) => r.kind === 'confirm');

  return (
    <article className="case">
      <header className="case-head">
        <p className="eyebrow">
          Case {c.id}
          {c.report.source_url && (
            <>
              {' · '}
              <a href={c.report.source_url}>source issue</a>
            </>
          )}
        </p>
        <h1 className="case-title">{c.report.title}</h1>
      </header>

      <section className="claim" aria-labelledby="claim-h">
        <h2 id="claim-h">The claim</h2>
        <blockquote>{(() => { const b = compactReport(c.report.body); return b.length > 900 ? b.slice(0, 900) + '…' : b; })()}</blockquote>
        {c.plan && c.plan.expected && (
          <p className="expect">
            <span>Expected, as Reenact read it</span>
            {c.plan.expected}
          </p>
        )}
      </section>

      <section className="reenact" aria-labelledby="re-h">
        <h2 id="re-h">The reenactment</h2>
        {hasStrip && (
          <div className="strip-wrap">
            <img src={caseFile(c.id, repro.run.strip_file ?? 'strip.png')} alt="Screenshots from Kane's run, left to right. The last one is the result." />
          </div>
        )}
        <Steps run={repro} />
      </section>

      <section className="verdict" aria-labelledby="v-h">
        <h2 id="v-h" className="sr">
          Verdict
        </h2>
        <Stamp kind={v.kind} big />
        <p className="verdict-line">{v.headline}</p>
        <dl className="facts">
          {v.stuck_at && (
            <>
              <dt>Where Kane stopped</dt>
              <dd>{v.stuck_at}</dd>
            </>
          )}
          {v.root_cause && (
            <>
              <dt>Kane's read on the cause</dt>
              <dd>{v.root_cause}</dd>
            </>
          )}
          {v.severity && (
            <>
              <dt>Severity</dt>
              <dd>{v.severity}</dd>
            </>
          )}
          {confirm && (
            <>
              <dt>Second run</dt>
              <dd>
                {confirm.verdict.kind === 'reproduced'
                  ? 'Reproduced again on an independent run (2 of 2) before any code was changed.'
                  : `Said "${VERDICT_LABEL[confirm.verdict.kind]}", so no fix was attempted.`}
              </dd>
            </>
          )}
          <dt>Cost</dt>
          <dd>
            {repro.run.credits.toFixed(0)} credits, {repro.run.duration_s.toFixed(0)} s
          </dd>
          {repro.run.share_url && (
            <>
              <dt>Evidence</dt>
              <dd>
                <a href={repro.run.share_url}>Kane run in Test Manager</a>
              </dd>
            </>
          )}
        </dl>
        {(repro.run.signals.console_errors.length > 0 || repro.run.signals.failed_requests.length > 0) && (
          <details>
            <summary>Browser signals during the run</summary>
            <ul className="signals">
              {repro.run.signals.console_errors.map((e, i) => (
                <li key={'c' + i}>
                  <b>console</b> {e}
                </li>
              ))}
              {repro.run.signals.failed_requests.map((e, i) => (
                <li key={'r' + i}>
                  <b>request</b> {e}
                </li>
              ))}
            </ul>
          </details>
        )}
      </section>

      {fix && (
        <section className="fix" aria-labelledby="fix-h">
          <h2 id="fix-h">The fix</h2>
          <p className="fix-line">
            {fixed ? (
              <>
                The same test now <strong className="ok">passes</strong>.
              </>
            ) : (
              <>
                After the attempted fix the result is <strong>{fix.verdict.kind.replace('_', ' ')}</strong>.
              </>
            )}{' '}
            {c.pr_url && (
              <a href={c.pr_url}>Pull request</a>
            )}
          </p>
          {fixed && <Compare before={caseFile(c.id, 'before.png')} after={caseFile(c.id, 'after.png')} />}
          <ol className="attempts">
            {c.fixes.map((a) => (
              <li key={a.attempt}>
                <span className="n">Attempt {a.attempt}</span>
                <p>{a.rationale}</p>
                <p className="attempt-note">
                  {a.files.length > 0 && <code>{a.files.join(', ')}</code>} {a.note}
                </p>
                {(a.suite_checked ?? 0) > 0 && (a.regressions?.length ?? 0) === 0 && (
                  <p className="attempt-note">{a.suite_checked} earlier saved test(s) re-run live and still hold.</p>
                )}
              </li>
            ))}
          </ol>
        </section>
      )}

      {c.runs.length > 1 && (
        <section className="history" aria-labelledby="h-h">
          <h2 id="h-h">Run history</h2>
          <ol className="doing">
            {c.runs.map((r, i) => (
              <li key={i}>
                <span className="n">{String(i + 1).padStart(2, '0')}</span>
                <span className="what">
                  {r.label} · {VERDICT_LABEL[r.verdict.kind]} · {new Date(r.started_at).toLocaleString()}
                </span>
                <span className="res">{r.run.credits.toFixed(0)} cr</span>
              </li>
            ))}
          </ol>
        </section>
      )}

      {c.test_md && (
        <section className="testfile" aria-labelledby="t-h">
          <h2 id="t-h">The test Kane ran</h2>
          <p className="muted">Plain English. Kept in the repo, re-run live with <code>reenact suite</code>.</p>
          <pre>{c.test_md}</pre>
        </section>
      )}
    </article>
  );
}
