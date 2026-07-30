import type { HalalStatus } from '../types';
import { halalStatusColor, halalStatusLabel } from '../utils';
import './HalalBadge.css';

interface HalalBadgeProps {
  status: HalalStatus;
  score: number;
  compact?: boolean;
}

export function HalalBadge({ status, score, compact }: HalalBadgeProps) {
  return (
    <div
      className={`halal-badge ${compact ? 'halal-badge--compact' : ''}`}
      style={{ '--badge-color': halalStatusColor(status) } as React.CSSProperties}
    >
      <span className="halal-badge__score">{score}</span>
      {!compact && <span className="halal-badge__label">{halalStatusLabel(status)}</span>}
    </div>
  );
}
