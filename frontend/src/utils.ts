import type { HalalStatus, HalalClassification, CertBody } from './types';

export function halalStatusLabel(status: HalalStatus): string {
  switch (status) {
    case 'halal': return 'Certified Halal';
    case 'likely_halal': return 'Likely Halal';
    case 'mashbooh': return 'Mashbooh';
    case 'likely_haram': return 'Likely Not Halal';
    case 'haram': return 'Not Halal';
    default: {
      const _exhaustive: never = status;
      return _exhaustive;
    }
  }
}

export function halalStatusColor(status: HalalStatus): string {
  switch (status) {
    case 'halal': return '#68a691';
    case 'likely_halal': return '#7db8a5';
    case 'mashbooh': return '#bfd3c1';
    case 'likely_haram': return '#c9a0a8';
    case 'haram': return '#694f5d';
    default: {
      const _exhaustive: never = status;
      return _exhaustive;
    }
  }
}

export function classificationColor(c: HalalClassification): string {
  switch (c) {
    case 'halal': return '#68a691';
    case 'mashbooh': return '#bfd3c1';
    case 'haram': return '#694f5d';
    default: {
      const _exhaustive: never = c;
      return _exhaustive;
    }
  }
}

export function certBodyLabel(body: CertBody): string {
  switch (body) {
    case 'IFANCA': return 'IFANCA';
    case 'AFIC': return 'AFIC';
    case 'LPPOM_MUI': return 'LPPOM MUI';
    case 'HCE': return 'HCE';
    default: {
      const _exhaustive: never = body;
      return _exhaustive;
    }
  }
}

export function formatPrice(gbp?: number): string {
  if (gbp == null) return '—';
  return `£${gbp.toFixed(2)}`;
}

export function formatSubType(subType: string): string {
  return subType.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

export function retailerLabel(retailer: string): string {
  switch (retailer) {
    case 'boots': return 'Boots';
    case 'superdrug': return 'Superdrug';
    case 'look_fantastic': return 'Look Fantastic';
    default: return retailer;
  }
}
