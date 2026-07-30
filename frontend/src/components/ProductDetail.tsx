import type { Product } from '../types';
import { HalalBadge } from './HalalBadge';
import { CertBadge } from './CertBadge';
import { classificationColor, formatPrice, formatSubType, halalStatusLabel, retailerLabel } from '../utils';
import './ProductDetail.css';

interface ProductDetailProps {
  product: Product;
  onClose: () => void;
}

export function ProductDetail({ product, onClose }: ProductDetailProps) {
  return (
    <div className="product-detail-overlay" onClick={onClose}>
      <div className="product-detail" onClick={e => e.stopPropagation()}>
        <button className="product-detail__close" onClick={onClose} aria-label="Close">
          ✕
        </button>

        <div className="product-detail__layout">
          <div className="product-detail__image-section">
            <img src={product.image_url} alt={product.name} className="product-detail__image" />
            <HalalBadge status={product.halal_status} score={product.halal_score} />
          </div>

          <div className="product-detail__info">
            <span className="product-detail__brand">{product.brand}</span>
            <h2 className="product-detail__name">{product.name}</h2>

            <div className="product-detail__meta-row">
              <span>{formatSubType(product.sub_type)}</span>
              <span>·</span>
              <span>{retailerLabel(product.retailer)}</span>
              <span>·</span>
              <span className="product-detail__price">{formatPrice(product.price_gbp)}</span>
            </div>

            {product.certifications.length > 0 && (
              <div className="product-detail__certs">
                <h4>Official Certification</h4>
                <div className="product-detail__cert-list">
                  {product.certifications.map(cert => (
                    <CertBadge key={cert.cert_number} certification={cert} />
                  ))}
                </div>
                {product.certifications[0].expiry_date && (
                  <p className="product-detail__cert-expiry">
                    Valid until {new Date(product.certifications[0].expiry_date).toLocaleDateString('en-GB')}
                  </p>
                )}
              </div>
            )}

            <div className="product-detail__score-section">
              <h4>Halal Analysis</h4>
              <div className="product-detail__score-bar">
                <div
                  className="product-detail__score-fill"
                  style={{ width: `${product.halal_score}%` }}
                />
              </div>
              <p className="product-detail__score-label">
                {product.halal_score}/100 — {halalStatusLabel(product.halal_status)}
              </p>
            </div>

            {product.halal_flags.length > 0 && (
              <div className="product-detail__flags">
                <h4>Flagged Ingredients</h4>
                {product.halal_flags.map(flag => (
                  <div key={flag.ingredient} className="product-detail__flag">
                    <span
                      className="product-detail__flag-dot"
                      style={{ background: classificationColor(flag.classification) }}
                    />
                    <div>
                      <strong>{flag.ingredient}</strong>
                      <p>{flag.reason}</p>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <div className="product-detail__ingredients">
              <h4>Ingredients (CosIng EU)</h4>
              {product.ingredients.map(ing => (
                <div key={ing.inci_name} className="product-detail__ingredient">
                  <span
                    className="product-detail__ingredient-dot"
                    style={{ background: classificationColor(ing.halal_classification) }}
                  />
                  <span className="product-detail__ingredient-name">{ing.inci_name}</span>
                  {ing.cas_number && (
                    <span className="product-detail__ingredient-cas">CAS: {ing.cas_number}</span>
                  )}
                </div>
              ))}
            </div>

            <a href={product.url} target="_blank" rel="noopener noreferrer" className="product-detail__link">
              View on {retailerLabel(product.retailer)} →
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
