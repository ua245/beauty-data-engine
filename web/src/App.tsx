import { useEffect, useMemo, useState } from "react";
import type { Catalog, CatalogProduct, Verdict } from "./types";
import {
  formatPrice,
  ingredientStatusLabel,
  summariseAssessment,
  verdictClass,
  verdictLabel,
} from "./lib/format";

const VERDICT_OPTIONS: { value: Verdict | "all"; label: string }[] = [
  { value: "all", label: "All verdicts" },
  { value: "certified-halal", label: "Certified halal" },
  { value: "likely-halal", label: "Likely halal" },
  { value: "needs-verification", label: "Needs verification" },
  { value: "likely-not-permissible", label: "Likely not permissible" },
  { value: "not-permissible", label: "Not permissible" },
  { value: "unknown", label: "Insufficient data" },
];

function ProductDrawer({
  item,
  onClose,
}: {
  item: CatalogProduct;
  onClose: () => void;
}) {
  const { product, assessment } = item;

  return (
    <>
      <div className="drawer-backdrop" onClick={onClose} aria-hidden />
      <aside className="drawer" role="dialog" aria-label={product.name}>
        <div className="drawer-header">
          <button type="button" onClick={onClose}>
            ← Back to catalogue
          </button>
          <div className="drawer-hero">
            {product.image ? (
              <img src={product.image.url} alt={product.image.alt} />
            ) : (
              <div
                style={{
                  width: 120,
                  height: 120,
                  borderRadius: 12,
                  background: "var(--ash-grey)",
                }}
              />
            )}
            <div>
              <div className="product-meta">
                <span>{product.retailer.name}</span>
                <span>{product.taxonomy.product_type_id.replace(/-/g, " ")}</span>
              </div>
              <h2 style={{ margin: "6px 0", fontFamily: "var(--font-serif)", fontWeight: 400 }}>
                {product.name}
              </h2>
              <p style={{ margin: 0, color: "var(--text-muted)" }}>{product.brand}</p>
              <div className="badge-row" style={{ marginTop: 10 }}>
                <span className={`badge ${verdictClass(assessment.verdict)}`}>
                  {verdictLabel(assessment.verdict)}
                </span>
                {assessment.certification !== "no-claim" && (
                  <span className="badge certified">{assessment.certification.replace(/-/g, " ")}</span>
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="score-bar" style={{ marginBottom: 16 }}>
          <label>
            <span>Halal likelihood</span>
            <strong>{assessment.likelihood}%</strong>
          </label>
          <div className="meter">
            <span style={{ width: `${assessment.likelihood}%` }} />
          </div>
        </div>
        <div className="score-bar" style={{ marginBottom: 20 }}>
          <label>
            <span>Assessment confidence</span>
            <strong>{assessment.confidence}%</strong>
          </label>
          <div className="meter">
            <span style={{ width: `${assessment.confidence}%` }} />
          </div>
        </div>

        {assessment.drivers.length > 0 && (
          <p style={{ fontSize: "0.9rem", lineHeight: 1.5 }}>
            <strong>Key drivers:</strong> {assessment.drivers.join(", ")}
          </p>
        )}

        {assessment.notes.length > 0 && (
          <div className="note-list">
            {assessment.notes.map((note) => (
              <div
                key={note.message}
                className={`note${note.severity === "blocker" ? " blocker" : ""}`}
              >
                {note.message}
              </div>
            ))}
          </div>
        )}

        <h3 style={{ fontFamily: "var(--font-serif)", fontWeight: 400 }}>Ingredients</h3>
        {product.ingredients_raw ? (
          <p style={{ fontSize: "0.85rem", color: "var(--text-muted)", lineHeight: 1.5 }}>
            {product.ingredients_raw}
          </p>
        ) : (
          <p className="note">No ingredient declaration on file for this sample.</p>
        )}

        <table className="ingredient-table">
          <thead>
            <tr>
              <th>INCI</th>
              <th>Status</th>
              <th>CosIng</th>
            </tr>
          </thead>
          <tbody>
            {assessment.ingredients.map((ing) => (
              <tr key={`${ing.position}-${ing.normalised}`}>
                <td>
                  <strong>{ing.normalised}</strong>
                  {ing.may_contain && (
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>
                      may contain
                    </div>
                  )}
                  {ing.concern && (
                    <div style={{ fontSize: "0.78rem", marginTop: 4, color: "var(--text-muted)" }}>
                      {ing.concern}
                    </div>
                  )}
                </td>
                <td>{ingredientStatusLabel(ing.status)}</td>
                <td>
                  {ing.cosing ? (
                    <>
                      <div>{ing.cosing.inci_name}</div>
                      {ing.cosing.annex_reference && (
                        <div style={{ fontSize: "0.75rem" }}>Annex {ing.cosing.annex_reference}</div>
                      )}
                    </>
                  ) : (
                    "—"
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        <p style={{ marginTop: 18, fontSize: "0.82rem" }}>
          <a href={product.url} target="_blank" rel="noreferrer">
            View on {product.retailer.name} →
          </a>
          {" · "}
          {formatPrice(product.price?.minor_units, product.price?.currency)}
        </p>
      </aside>
    </>
  );
}

export default function App() {
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [retailer, setRetailer] = useState("all");
  const [category, setCategory] = useState("all");
  const [verdict, setVerdict] = useState<Verdict | "all">("all");
  const [selected, setSelected] = useState<CatalogProduct | null>(null);

  useEffect(() => {
    fetch("/data/catalog.json")
      .then((r) => {
        if (!r.ok) throw new Error(`Failed to load catalogue (${r.status})`);
        return r.json();
      })
      .then((data: Catalog) => setCatalog(data))
      .catch((e: Error) => setError(e.message));
  }, []);

  const categories = useMemo(() => {
    if (!catalog) return [];
    const set = new Set(catalog.products.map((p) => p.product.taxonomy.department_id));
    return Array.from(set).sort();
  }, [catalog]);

  const filtered = useMemo(() => {
    if (!catalog) return [];
    const q = query.trim().toLowerCase();
    return catalog.products.filter(({ product, assessment }) => {
      if (retailer !== "all" && product.retailer.id !== retailer) return false;
      if (category !== "all" && product.taxonomy.department_id !== category) return false;
      if (verdict !== "all" && assessment.verdict !== verdict) return false;
      if (!q) return true;
      const hay = `${product.name} ${product.brand} ${product.ingredients_raw ?? ""}`.toLowerCase();
      return hay.includes(q);
    });
  }, [catalog, query, retailer, category, verdict]);

  const stats = useMemo(() => {
    if (!catalog) return null;
    const total = catalog.products.length;
    const certified = catalog.products.filter(
      (p) => p.assessment.verdict === "certified-halal",
    ).length;
    const blocked = catalog.products.filter((p) =>
      ["not-permissible", "likely-not-permissible"].includes(p.assessment.verdict),
    ).length;
    const withIngredients = catalog.products.filter((p) => p.product.ingredients_raw).length;
    return { total, certified, blocked, withIngredients };
  }, [catalog]);

  if (error) {
    return (
      <div className="app-shell">
        <div className="empty">Could not load catalogue: {error}</div>
      </div>
    );
  }

  if (!catalog || !stats) {
    return (
      <div className="app-shell">
        <div className="empty">Loading sample catalogue…</div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="hero">
        <div className="hero-inner">
          <span className="eyebrow">Halal Beauty Engine · Sample dashboard</span>
          <h1>Find halal beauty with evidence, not guesswork.</h1>
          <p>
            UK retailer products enriched with CosIng ingredient data, halal likelihood scoring,
            certification trust-ladder checks, and wudu-compatibility notes. This deployment uses
            bundled sample data — wire up the Rust scraper for live ingestion.
          </p>
        </div>
      </header>

      <section className="stats-row">
        <div className="stat-card">
          <strong>{stats.total}</strong>
          <span>Sample products</span>
        </div>
        <div className="stat-card">
          <strong>{stats.certified}</strong>
          <span>Certified halal</span>
        </div>
        <div className="stat-card">
          <strong>{stats.blocked}</strong>
          <span>Flagged products</span>
        </div>
        <div className="stat-card">
          <strong>{stats.withIngredients}</strong>
          <span>With INCI lists</span>
        </div>
      </section>

      <section className="toolbar">
        <label>
          Search
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Brand, product, ingredient…"
          />
        </label>
        <label>
          Retailer
          <select value={retailer} onChange={(e) => setRetailer(e.target.value)}>
            <option value="all">All retailers</option>
            {catalog.retailers.map((r) => (
              <option key={r.id} value={r.id}>
                {r.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Department
          <select value={category} onChange={(e) => setCategory(e.target.value)}>
            <option value="all">All departments</option>
            {categories.map((c) => (
              <option key={c} value={c}>
                {c.replace(/-/g, " ")}
              </option>
            ))}
          </select>
        </label>
        <label>
          Verdict
          <select
            value={verdict}
            onChange={(e) => setVerdict(e.target.value as Verdict | "all")}
          >
            {VERDICT_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
      </section>

      {filtered.length === 0 ? (
        <div className="empty">No products match your filters.</div>
      ) : (
        <section className="product-grid">
          {filtered.map((item) => {
            const { product, assessment } = item;
            return (
              <article
                key={product.id}
                className="product-card"
                onClick={() => setSelected(item)}
                onKeyDown={(e) => e.key === "Enter" && setSelected(item)}
                role="button"
                tabIndex={0}
              >
                {product.image ? (
                  <img src={product.image.url} alt={product.image.alt} loading="lazy" />
                ) : (
                  <div />
                )}
                <div className="product-card-body">
                  <div className="product-meta">
                    <span>{product.retailer.name}</span>
                    <span>{formatPrice(product.price?.minor_units, product.price?.currency)}</span>
                  </div>
                  <h3>{product.name}</h3>
                  <div className="badge-row">
                    <span className={`badge ${verdictClass(assessment.verdict)}`}>
                      {verdictLabel(assessment.verdict)}
                    </span>
                  </div>
                  <div className="score-bar">
                    <label>
                      <span>Likelihood</span>
                      <span>{assessment.likelihood}%</span>
                    </label>
                    <div className="meter">
                      <span style={{ width: `${assessment.likelihood}%` }} />
                    </div>
                  </div>
                  <p style={{ margin: 0, fontSize: "0.78rem", color: "var(--text-muted)" }}>
                    {summariseAssessment(assessment)}
                  </p>
                </div>
              </article>
            );
          })}
        </section>
      )}

      {selected && <ProductDrawer item={selected} onClose={() => setSelected(null)} />}
    </div>
  );
}
