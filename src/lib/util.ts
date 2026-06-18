import type { Diagnostic } from "../types";

/** Stable identity for a diagnostic, used for active-row highlighting. */
export function diagKey(d: Diagnostic): string {
  return `${d.code}|${d.path}|${d.range?.startLine ?? 0}|${d.range?.startCol ?? 0}`;
}
