import type { Certification } from '../types';
import { certBodyLabel } from '../utils';
import './CertBadge.css';

interface CertBadgeProps {
  certification: Certification;
}

export function CertBadge({ certification }: CertBadgeProps) {
  return (
    <div className="cert-badge" title={`${certBodyLabel(certification.body)} — ${certification.cert_number}`}>
      <span className="cert-badge__body">{certBodyLabel(certification.body)}</span>
      <span className="cert-badge__status">✓ Certified</span>
    </div>
  );
}
