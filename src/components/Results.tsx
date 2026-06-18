import type { Diagnostic, Severity, ValidationReport } from "../types";
import { openExternal } from "../lib/api";
import { diagKey } from "../lib/util";

interface Props {
  report: ValidationReport | null;
  filters: Record<Severity, boolean>;
  activeKey: string | null;
  onToggleFilter: (s: Severity) => void;
  onJump: (d: Diagnostic) => void;
  onCopy: () => void;
}

const SEV_LABEL: Record<Severity, string> = {
  error: "Errors",
  warning: "Warnings",
  info: "Info",
};

const SEV_ICON: Record<Severity, string> = {
  error: "✖",
  warning: "▲",
  info: "ⓘ",
};

export function Results(props: Props) {
  const { report, filters, activeKey, onToggleFilter, onJump, onCopy } = props;

  if (!report) {
    return (
      <div className="results">
        <div className="results-empty">검증을 실행하면 결과가 여기에 표시됩니다.</div>
      </div>
    );
  }

  const order: Severity[] = ["error", "warning", "info"];
  const counts: Record<Severity, number> = {
    error: report.summary.errors,
    warning: report.summary.warnings,
    info: report.summary.infos,
  };

  return (
    <div className="results">
      <div className="results-header">
        <div className="filter-row">
          {order.map((s) => (
            <button
              key={s}
              className={`filter-chip ${s} ${filters[s] ? "on" : "off"}`}
              onClick={() => onToggleFilter(s)}
              title={`${SEV_LABEL[s]} 표시 토글`}
            >
              {SEV_ICON[s]} {counts[s]}
            </button>
          ))}
        </div>
        <button className="copy-btn" onClick={onCopy} title="결과를 클립보드로 복사">
          복사
        </button>
      </div>

      {report.ok && report.summary.warnings === 0 && report.summary.infos === 0 && (
        <div className="results-ok">✓ 유효한 table-mapping 문서입니다.</div>
      )}

      <div className="results-list">
        {order.map((sev) =>
          filters[sev]
            ? report.diagnostics
                .filter((d) => d.severity === sev)
                .map((d, i) => {
                  const key = diagKey(d);
                  return (
                    <DiagRow
                      key={`${key}#${i}`}
                      diag={d}
                      active={activeKey === key}
                      onJump={onJump}
                    />
                  );
                })
            : null,
        )}
      </div>
    </div>
  );
}

function DiagRow({
  diag,
  active,
  onJump,
}: {
  diag: Diagnostic;
  active: boolean;
  onJump: (d: Diagnostic) => void;
}) {
  const loc = diag.range ? `${diag.range.startLine}:${diag.range.startCol}` : "";
  return (
    <div
      className={`diag-row ${diag.severity} ${active ? "active" : ""}`}
      onClick={() => onJump(diag)}
      title={`${diag.messageKo}\n${diag.message}${diag.docUrl ? `\n\n${diag.docUrl}` : ""}`}
    >
      <div className="diag-line1">
        <span className={`sev-dot ${diag.severity}`}>{SEV_ICON[diag.severity]}</span>
        <span className="diag-code">{diag.code}</span>
        <span className="diag-loc">{loc}</span>
        {diag.docUrl && (
          <button
            className="doc-link"
            title={diag.docUrl}
            onClick={(e) => {
              e.stopPropagation();
              if (diag.docUrl) void openExternal(diag.docUrl);
            }}
          >
            ↗ docs
          </button>
        )}
      </div>
      <div className="diag-msg-ko">{diag.messageKo}</div>
      <div className="diag-msg-en">{diag.message}</div>
      <div className="diag-path">{diag.path || "(root)"}</div>
    </div>
  );
}
