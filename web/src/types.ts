export type Verdict =
  | "not-permissible"
  | "likely-not-permissible"
  | "unknown"
  | "needs-verification"
  | "likely-halal"
  | "certified-halal";

export type HalalStatus = "halal" | "mashbooh" | "haram" | "unknown";

export interface Money {
  minor_units: number;
  currency: "GBP" | "EUR" | "USD";
}

export interface ProductImage {
  url: string;
  alt: string;
}

export interface TaxonomyAssignment {
  department_id: string;
  category_id: string;
  product_type_id: string;
  method: string;
  confidence: number;
}

export interface IngredientAssessment {
  position: number;
  raw: string;
  normalised: string;
  ci_number?: string;
  may_contain: boolean;
  status: HalalStatus;
  rule_id?: string;
  rule_label?: string;
  concern?: string;
  residual_risk: number;
  cosing?: {
    inci_name: string;
    cas_number?: string;
    functions: string[];
    annex_reference?: string;
  };
  wudu_barrier: boolean;
}

export interface HalalAssessment {
  profile_id: string;
  verdict: Verdict;
  likelihood: number;
  confidence: number;
  wudu: string;
  certification: string;
  ingredients: IngredientAssessment[];
  notes: { severity: string; message: string }[];
  drivers: string[];
  unrecognised_count: number;
}

export interface Product {
  id: string;
  name: string;
  brand: string;
  retailer: { id: string; name: string; domain: string };
  url: string;
  shade?: string;
  size?: string;
  price?: Money;
  image?: ProductImage;
  taxonomy: TaxonomyAssignment;
  ingredients_raw?: string;
  claims: string[];
}

export interface CatalogProduct {
  product: Product;
  assessment: HalalAssessment;
}

export interface Catalog {
  version: string;
  generated_at: string;
  profile: string;
  products: CatalogProduct[];
  retailers: { id: string; name: string; domain: string }[];
}
