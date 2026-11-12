import { useState } from 'react';

/** Before (red) and after (green) on the same frame, with a draggable divider. */
export function Compare({ before, after }: { before: string; after: string }) {
  const [pos, setPos] = useState(50);
  return (
    <figure className="compare">
      <div className="compare-stage">
        <img src={after} alt="After the fix: the same screen, behaving as expected" />
        <img
          className="compare-before"
          src={before}
          alt="Before the fix: the reported behavior"
          style={{ clipPath: `inset(0 ${100 - pos}% 0 0)` }}
        />
        <div className="compare-rule" style={{ left: `${pos}%` }} aria-hidden="true" />
        <span className="compare-tag compare-tag-before">Before</span>
        <span className="compare-tag compare-tag-after">After</span>
      </div>
      <input
        type="range"
        min={0}
        max={100}
        value={pos}
        onChange={(e) => setPos(Number(e.target.value))}
        aria-label="Reveal before and after"
      />
    </figure>
  );
}
