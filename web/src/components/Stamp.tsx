import { VERDICT_LABEL, type VerdictKind } from '../types';

export function Stamp({ kind, big = false }: { kind: VerdictKind; big?: boolean }) {
  return <span className={`stamp stamp-${kind}${big ? ' stamp-big' : ''}`}>{VERDICT_LABEL[kind]}{big ? '.' : ''}</span>;
}

export function kindFromLabel(label: string): VerdictKind {
  switch (label.toLowerCase()) {
    case 'reproduced':
      return 'reproduced';
    case 'not reproduced':
      return 'not_reproduced';
    case 'needs info':
      return 'needs_info';
    default:
      return 'inconclusive';
  }
}
