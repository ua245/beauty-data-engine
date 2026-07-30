export type HalalStatus = 'halal' | 'likely_halal' | 'mashbooh' | 'likely_haram' | 'haram';
export type ProductType = 'makeup' | 'skincare' | 'fragrance' | 'haircare';
export type Retailer = 'boots' | 'superdrug' | 'look_fantastic';
export type CertBody = 'IFANCA' | 'AFIC' | 'LPPOM_MUI' | 'HCE';
export type HalalClassification = 'halal' | 'mashbooh' | 'haram';

export interface CosingIngredient {
  inci_name: string;
  slug?: string;
  cas_number?: string;
  ec_number?: string;
  functions: string[];
  restriction?: string;
  halal_classification: HalalClassification;
}

export interface HalalFlag {
  ingredient: string;
  classification: HalalClassification;
  reason: string;
  penalty: number;
}

export interface Certification {
  body: CertBody;
  cert_number: string;
  brand: string;
  product_names: string[];
  issue_date?: string;
  expiry_date?: string;
  status: 'active' | 'expired' | 'revoked' | 'suspended';
  inspection_body?: string;
  confidence: number;
}

export interface Product {
  id: string;
  retailer: Retailer;
  retailer_sku: string;
  name: string;
  brand: string;
  image_url: string;
  category_path: string;
  product_type: ProductType;
  sub_type: string;
  ingredients_raw?: string;
  ingredients: CosingIngredient[];
  halal_score: number;
  halal_status: HalalStatus;
  halal_flags: HalalFlag[];
  certifications: Certification[];
  price_gbp?: number;
  url: string;
  scraped_at: string;
}

export interface ProductFilters {
  query?: string;
  product_type?: ProductType[];
  sub_type?: string[];
  retailer?: Retailer[];
  halal_status?: HalalStatus[];
  certified_only?: boolean;
  cert_body?: CertBody[];
  price_min?: number;
  price_max?: number;
  page?: number;
  per_page?: number;
}

export interface ProductListResponse {
  products: Product[];
  total: number;
  page: number;
  per_page: number;
}

export interface TaxonomyNode {
  slug: string;
  label: string;
  path: string;
  children: TaxonomyNode[];
}
