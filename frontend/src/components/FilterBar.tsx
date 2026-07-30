import type { HalalStatus, ProductType } from '../types';
import './FilterBar.css';

interface FilterBarProps {
  query: string;
  onQueryChange: (q: string) => void;
  productType: ProductType | '';
  onProductTypeChange: (t: ProductType | '') => void;
  halalStatus: HalalStatus | '';
  onHalalStatusChange: (s: HalalStatus | '') => void;
  certifiedOnly: boolean;
  onCertifiedOnlyChange: (v: boolean) => void;
  total: number;
}

const PRODUCT_TYPES: { value: ProductType | ''; label: string }[] = [
  { value: '', label: 'All Categories' },
  { value: 'makeup', label: 'Makeup' },
  { value: 'fragrance', label: 'Fragrance' },
  { value: 'haircare', label: 'Haircare' },
  { value: 'skincare', label: 'Skincare' },
];

const HALAL_STATUSES: { value: HalalStatus | ''; label: string }[] = [
  { value: '', label: 'All Statuses' },
  { value: 'halal', label: 'Certified Halal' },
  { value: 'likely_halal', label: 'Likely Halal' },
  { value: 'mashbooh', label: 'Mashbooh' },
  { value: 'likely_haram', label: 'Likely Not Halal' },
  { value: 'haram', label: 'Not Halal' },
];

export function FilterBar({
  query,
  onQueryChange,
  productType,
  onProductTypeChange,
  halalStatus,
  onHalalStatusChange,
  certifiedOnly,
  onCertifiedOnlyChange,
  total,
}: FilterBarProps) {
  return (
    <div className="filter-bar">
      <div className="filter-bar__search">
        <input
          type="search"
          placeholder="Search products, brands..."
          value={query}
          onChange={e => onQueryChange(e.target.value)}
          className="filter-bar__input"
        />
      </div>

      <div className="filter-bar__filters">
        <select
          value={productType}
          onChange={e => onProductTypeChange(e.target.value as ProductType | '')}
          className="filter-bar__select"
        >
          {PRODUCT_TYPES.map(t => (
            <option key={t.value} value={t.value}>{t.label}</option>
          ))}
        </select>

        <select
          value={halalStatus}
          onChange={e => onHalalStatusChange(e.target.value as HalalStatus | '')}
          className="filter-bar__select"
        >
          {HALAL_STATUSES.map(s => (
            <option key={s.value} value={s.value}>{s.label}</option>
          ))}
        </select>

        <label className="filter-bar__checkbox">
          <input
            type="checkbox"
            checked={certifiedOnly}
            onChange={e => onCertifiedOnlyChange(e.target.checked)}
          />
          Certified only
        </label>
      </div>

      <span className="filter-bar__count">{total} products</span>
    </div>
  );
}
