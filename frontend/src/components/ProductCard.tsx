import type { Product } from '../types';
import { HalalBadge } from './HalalBadge';
import { CertBadge } from './CertBadge';
import { formatPrice, formatSubType, retailerLabel } from '../utils';
import './ProductCard.css';

interface ProductCardProps {
  product: Product;
  onSelect: (product: Product) => void;
}

export function ProductCard({ product, onSelect }: ProductCardProps) {
  return (
    <article className="product-card" onClick={() => onSelect(product)}>
      <div className="product-card__image-wrap">
        <img
          src={product.image_url}
          alt={product.name}
          className="product-card__image"
          loading="lazy"
        />
        <div className="product-card__badge-overlay">
          <HalalBadge status={product.halal_status} score={product.halal_score} compact />
        </div>
      </div>

      <div className="product-card__body">
        <div className="product-card__meta">
          <span className="product-card__brand">{product.brand}</span>
          <span className="product-card__retailer">{retailerLabel(product.retailer)}</span>
        </div>

        <h3 className="product-card__name">{product.name}</h3>

        <div className="product-card__tags">
          <span className="product-card__tag">{formatSubType(product.sub_type)}</span>
          {product.certifications.length > 0 && (
            <CertBadge certification={product.certifications[0]} />
          )}
        </div>

        <div className="product-card__footer">
          <span className="product-card__price">{formatPrice(product.price_gbp)}</span>
          <span className="product-card__ingredients">
            {product.ingredients.length} ingredients
          </span>
        </div>
      </div>
    </article>
  );
}
