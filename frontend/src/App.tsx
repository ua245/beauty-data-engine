import { useCallback, useEffect, useState } from 'react';
import { fetchProducts } from './api';
import type { Product, ProductType, HalalStatus } from './types';
import { ProductCard } from './components/ProductCard';
import { ProductDetail } from './components/ProductDetail';
import { FilterBar } from './components/FilterBar';
import './App.css';

function App() {
  const [products, setProducts] = useState<Product[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<Product | null>(null);

  const [query, setQuery] = useState('');
  const [productType, setProductType] = useState<ProductType | ''>('');
  const [halalStatus, setHalalStatus] = useState<HalalStatus | ''>('');
  const [certifiedOnly, setCertifiedOnly] = useState(false);

  const loadProducts = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchProducts({
        query: query || undefined,
        product_type: productType ? [productType] : undefined,
        halal_status: halalStatus ? [halalStatus] : undefined,
        certified_only: certifiedOnly || undefined,
        per_page: 50,
      });
      setProducts(data.products);
      setTotal(data.total);
    } catch {
      setProducts([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  }, [query, productType, halalStatus, certifiedOnly]);

  useEffect(() => {
    loadProducts();
  }, [loadProducts]);

  return (
    <div className="app">
      <header className="app__header">
        <div className="app__header-content">
          <h1 className="app__title">Halal Beauty Engine</h1>
          <p className="app__subtitle">
            UK beauty products analysed against EU CosIng ingredients &amp; halal certification
          </p>
        </div>
        <div className="app__header-accent" />
      </header>

      <main className="app__main">
        <FilterBar
          query={query}
          onQueryChange={setQuery}
          productType={productType}
          onProductTypeChange={setProductType}
          halalStatus={halalStatus}
          onHalalStatusChange={setHalalStatus}
          certifiedOnly={certifiedOnly}
          onCertifiedOnlyChange={setCertifiedOnly}
          total={total}
        />

        {loading ? (
          <div className="app__loading">
            <div className="app__spinner" />
            <p>Analysing products...</p>
          </div>
        ) : products.length === 0 ? (
          <div className="app__empty">
            <p>No products match your filters.</p>
          </div>
        ) : (
          <div className="app__grid">
            {products.map(product => (
              <ProductCard
                key={product.id}
                product={product}
                onSelect={setSelected}
              />
            ))}
          </div>
        )}
      </main>

      <footer className="app__footer">
        <p>
          Ingredient analysis is informational, not a religious ruling.
          Always verify with official certification from IFANCA, AFIC, LPPOM MUI, or HCE.
        </p>
      </footer>

      {selected && (
        <ProductDetail product={selected} onClose={() => setSelected(null)} />
      )}
    </div>
  );
}

export default App;
