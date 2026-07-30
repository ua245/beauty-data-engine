import type { HalalAssessment, Verdict } from "../types";

export function verdictLabel(verdict: Verdict): string {
  switch (verdict) {
    case "certified-halal":
      return "Certified halal";
    case "likely-halal":
      return "Likely halal";
    case "needs-verification":
      return "Needs verification";
    case "likely-not-permissible":
      return "Likely not permissible";
    case "not-permissible":
      return "Not permissible";
    case "unknown":
      return "Insufficient data";
    default: {
      const _exhaustive: never = verdict;
      return _exhaustive;
    }
  }
}

export function verdictClass(verdict: Verdict): string {
  switch (verdict) {
    case "certified-halal":
      return "certified";
    case "likely-halal":
      return "likely";
    case "needs-verification":
      return "verify";
    case "likely-not-permissible":
    case "not-permissible":
      return "blocked";
    case "unknown":
      return "unknown";
    default: {
      const _exhaustive: never = verdict;
      return _exhaustive;
    }
  }
}

export function formatPrice(minor?: number, currency = "GBP"): string {
  if (minor == null) return "—";
  return new Intl.NumberFormat("en-GB", {
    style: "currency",
    currency,
  }).format(minor / 100);
}

export function ingredientStatusLabel(status: string): string {
  switch (status) {
    case "halal":
      return "Halal";
    case "mashbooh":
      return "Doubtful";
    case "haram":
      return "Not permissible";
    case "unknown":
      return "Unknown";
    default:
      return status;
  }
}

export function summariseAssessment(a: HalalAssessment): string {
  return `${verdictLabel(a.verdict)} · ${a.likelihood}% likelihood · ${a.confidence}% confidence`;
}
