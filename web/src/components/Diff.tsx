/** A unified diff, with added and removed lines coloured. Hunks only, no file headers. */
export function Diff({ text, label }: { text: string; label: string }) {
  const lines = text
    .split('\n')
    .filter((l) => !l.startsWith('index ') && !l.startsWith('diff --git') && !l.startsWith('--- ') && !l.startsWith('+++ '));
  const files = text
    .split('\n')
    .filter((l) => l.startsWith('+++ '))
    .map((l) => l.slice(4).replace(/^b\//, ''));
  return (
    <figure className="diff">
      <figcaption>
        <span>{label}</span>
        {files.length > 0 && <code>{files.join(', ')}</code>}
      </figcaption>
      <pre aria-label={label}>
        {lines.map((l, i) => (
          <span key={i} className={l.startsWith('+') ? 'add' : l.startsWith('-') ? 'del' : l.startsWith('@@') ? 'hunk' : ''}>
            {l + '\n'}
          </span>
        ))}
      </pre>
    </figure>
  );
}
