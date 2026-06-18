// Mirrors the Rust `dms-core` model (SPEC §7), serialized as camelCase.

export type Severity = "error" | "warning" | "info";

export interface DiagRange {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
}

export interface RuleContext {
  ruleId?: string;
  ruleName?: string;
  ruleType?: string;
}

export interface Diagnostic {
  code: string;
  severity: Severity;
  message: string;
  messageKo: string;
  path: string;
  range?: DiagRange;
  ruleContext?: RuleContext;
  docUrl?: string;
}

export interface Summary {
  errors: number;
  warnings: number;
  infos: number;
}

export interface ValidationReport {
  ok: boolean;
  summary: Summary;
  engineVersion: string;
  targetDmsVersion: string;
  diagnostics: Diagnostic[];
}

export interface RuleDoc {
  code: string;
  severity: Severity;
  message: string;
  messageKo: string;
  docUrl?: string;
}
