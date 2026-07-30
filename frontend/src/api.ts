import type { Product, ProductFilters, ProductListResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? '/api/v1';

export async function fetchProducts(filters: ProductFilters = {}): Promise<ProductListResponse> {
  const params = new URLSearchParams();

  if (filters.query) params.set('query', filters.query);
  if (filters.product_type?.length) params.set('product_type', filters.product_type.join(','));
  if (filters.sub_type?.length) params.set('sub_type', filters.sub_type.join(','));
  if (filters.retailer?.length) params.set('retailer', filters.retailer.join(','));
  if (filters.halal_status?.length) params.set('halal_status', filters.halal_status.join(','));
  if (filters.certified_only) params.set('certified_only', 'true');
  if (filters.page) params.set('page', String(filters.page));
  if (filters.per_page) params.set('per_page', String(filters.per_page));

  const res = await fetch(`${API_BASE}/products?${params}`);
  if (!res.ok) throw new Error('Failed to fetch products');
  return res.json();
}

export async function fetchProduct(id: string): Promise<Product> {
  const res = await fetch(`${API_BASE}/products/${id}`);
  if (!res.ok) throw new Error('Product not found');
  return res.json();
}

export async function fetchTaxonomy() {
  const res = await fetch(`${API_BASE}/taxonomy`);
  if (!res.ok) throw new Error('Failed to fetch taxonomy');
  return res.json();
}
