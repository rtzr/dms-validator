import type { ValidationReport } from "../types";

const MAX_BYTES = 2 * 1024 * 1024;

interface Props {
  report: ValidationReport | null;
  byteSize: number;
  tauri: boolean;
}

export function StatusBar({ report, byteSize, tauri }: Props) {
  const kb = (byteSize / 1024).toFixed(1);
  const overLimit = byteSize > MAX_BYTES;

  return (
    <div className="statusbar">
      <div className="status-left">
        {report ? (
          <>
            <span className="status-err">✖ {report.summary.errors}</span>
            <span className="status-warn">▲ {report.summary.warnings}</span>
            <span className="status-info">ⓘ {report.summary.infos}</span>
            <span className={`status-state ${report.ok ? "ok" : "fail"}`}>
              {report.ok ? "OK" : "FAILED"}
            </span>
          </>
        ) : (
          <span className="status-muted">검증 대기</span>
        )}
      </div>
      <div className="status-right">
        <span className={overLimit ? "status-err" : "status-muted"}>
          {kb} KB / 2 MB
        </span>
        {report && <span className="status-muted">target DMS {report.targetDmsVersion}</span>}
        {report && <span className="status-muted">engine {report.engineVersion}</span>}
        <span className="status-muted">{tauri ? "Tauri" : "Browser (read-only)"}</span>
      </div>
    </div>
  );
}
