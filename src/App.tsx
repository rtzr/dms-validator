import { useCallback, useEffect, useRef, useState } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import type { monaco } from "./lib/monaco";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import {
  isTauri,
  readTextFile,
  validateDocument,
  writeTextFile,
} from "./lib/api";
import { diagKey } from "./lib/util";
import type { Diagnostic, Severity, ValidationReport } from "./types";
import { Toolbar } from "./components/Toolbar";
import { Results } from "./components/Results";
import { StatusBar } from "./components/StatusBar";
import { SAMPLES } from "./lib/samples";

type MonacoNs = typeof monaco;
type CodeEditor = monaco.editor.IStandaloneCodeEditor;

const DEBOUNCE_MS = 400;

export default function App() {
  const tauri = isTauri();
  const [content, setContent] = useState<string>(SAMPLES[1].content);
  const [report, setReport] = useState<ValidationReport | null>(null);
  const [version, setVersion] = useState<string>("3.5.4");
  const [live, setLive] = useState<boolean>(true);
  const [fileName, setFileName] = useState<string | null>(null);
  const [dirty, setDirty] = useState<boolean>(false);
  const [activeKey, setActiveKey] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [filters, setFilters] = useState<Record<Severity, boolean>>({
    error: true,
    warning: true,
    info: true,
  });

  const editorRef = useRef<CodeEditor | null>(null);
  const monacoRef = useRef<MonacoNs | null>(null);
  const contentRef = useRef(content);
  contentRef.current = content;

  const runValidate = useCallback(async () => {
    if (!isTauri()) {
      setNotice("검증 엔진은 데스크톱 앱에서 동작합니다 — `pnpm tauri dev`로 실행하세요.");
      return;
    }
    try {
      const r = await validateDocument(contentRef.current, version);
      setReport(r);
      setNotice(null);
    } catch (e) {
      setNotice(`검증 실패: ${String(e)}`);
    }
  }, [version]);

  // Live validation (debounced).
  useEffect(() => {
    if (!live) return;
    const t = setTimeout(() => void runValidate(), DEBOUNCE_MS);
    return () => clearTimeout(t);
  }, [content, version, live, runValidate]);

  // Validate once on startup.
  useEffect(() => {
    void runValidate();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Render diagnostics as editor markers.
  useEffect(() => {
    const editor = editorRef.current;
    const m = monacoRef.current;
    if (!editor || !m) return;
    const model = editor.getModel();
    if (!model) return;
    const markers = (report?.diagnostics ?? [])
      .filter((d) => d.range)
      .map((d) => ({
        startLineNumber: d.range!.startLine,
        startColumn: d.range!.startCol,
        endLineNumber: d.range!.endLine,
        endColumn: d.range!.endCol,
        message: `[${d.code}] ${d.messageKo}\n${d.message}`,
        severity: sevToMonaco(m, d.severity),
        source: "dms",
      }));
    m.editor.setModelMarkers(model, "dms", markers);
  }, [report]);

  const onMount: OnMount = (editor, m) => {
    editorRef.current = editor;
    monacoRef.current = m as MonacoNs;
  };

  const jump = useCallback((d: Diagnostic) => {
    const editor = editorRef.current;
    if (!editor || !d.range) return;
    const r = {
      startLineNumber: d.range.startLine,
      startColumn: d.range.startCol,
      endLineNumber: d.range.endLine,
      endColumn: d.range.endCol,
    };
    editor.revealRangeInCenterIfOutsideViewport(r);
    editor.setSelection(r);
    editor.focus();
    setActiveKey(diagKey(d));
  }, []);

  const loadContent = useCallback((text: string, name: string | null) => {
    setContent(text);
    setFileName(name);
    setDirty(false);
  }, []);

  const onOpen = useCallback(async () => {
    if (!tauri) return;
    try {
      const path = await openDialog({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (typeof path === "string") {
        const text = await readTextFile(path);
        loadContent(text, path);
      }
    } catch (e) {
      setNotice(`열기 실패: ${String(e)}`);
    }
  }, [tauri, loadContent]);

  const onSave = useCallback(async () => {
    if (!tauri) return;
    try {
      let path = fileName;
      if (!path) {
        const chosen = await saveDialog({
          defaultPath: "table-mappings.json",
          filters: [{ name: "JSON", extensions: ["json"] }],
        });
        if (!chosen) return;
        path = chosen;
      }
      await writeTextFile(path, content);
      setFileName(path);
      setDirty(false);
    } catch (e) {
      setNotice(`저장 실패: ${String(e)}`);
    }
  }, [tauri, fileName, content]);

  const onDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      if (!file) return;
      const text = await file.text();
      loadContent(text, file.name);
    },
    [loadContent],
  );

  const copyResults = useCallback(() => {
    if (!report) return;
    const lines = report.diagnostics.map(
      (d) =>
        `${d.severity.toUpperCase()}\t${d.code}\t${
          d.range ? `${d.range.startLine}:${d.range.startCol}` : ""
        }\t${d.message}\t${d.path}`,
    );
    const header = `# DMS validation — ${report.summary.errors} errors, ${report.summary.warnings} warnings, ${report.summary.infos} info (target DMS ${report.targetDmsVersion})`;
    void navigator.clipboard.writeText([header, ...lines].join("\n"));
  }, [report]);

  return (
    <div className="app">
      <Toolbar
        fileName={fileName}
        dirty={dirty}
        tauri={tauri}
        live={live}
        version={version}
        onOpen={onOpen}
        onSave={onSave}
        onLoadSample={(c) => loadContent(c, null)}
        onValidate={() => void runValidate()}
        onToggleLive={setLive}
        onVersionChange={setVersion}
      />

      {notice && (
        <div className="notice" onClick={() => setNotice(null)}>
          {notice}
        </div>
      )}

      <div className="main" onDragOver={(e) => e.preventDefault()} onDrop={onDrop}>
        <div className="editor-pane">
          <Editor
            language="json"
            theme="vs-dark"
            value={content}
            onMount={onMount}
            onChange={(v) => {
              setContent(v ?? "");
              setDirty(true);
            }}
            options={{
              fontSize: 13,
              minimap: { enabled: false },
              automaticLayout: true,
              scrollBeyondLastLine: false,
              tabSize: 2,
              renderWhitespace: "selection",
              fixedOverflowWidgets: true,
            }}
          />
        </div>
        <div className="results-pane">
          <Results
            report={report}
            filters={filters}
            activeKey={activeKey}
            onToggleFilter={(s) => setFilters((f) => ({ ...f, [s]: !f[s] }))}
            onJump={jump}
            onCopy={copyResults}
          />
        </div>
      </div>

      <StatusBar report={report} byteSize={byteLength(content)} tauri={tauri} />
    </div>
  );
}

function sevToMonaco(m: MonacoNs, s: Severity): monaco.MarkerSeverity {
  switch (s) {
    case "error":
      return m.MarkerSeverity.Error;
    case "warning":
      return m.MarkerSeverity.Warning;
    default:
      return m.MarkerSeverity.Info;
  }
}

function byteLength(s: string): number {
  return new TextEncoder().encode(s).length;
}
