// Wires Monaco's web workers through Vite so everything is bundled locally
// (no CDN fetch) — required for the offline desktop app.
import * as monaco from "monaco-editor";
import { loader } from "@monaco-editor/react";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";

declare global {
  // eslint-disable-next-line no-var
  interface Window {
    MonacoEnvironment?: monaco.Environment;
  }
}

self.MonacoEnvironment = {
  getWorker(_moduleId: string, label: string) {
    if (label === "json") return new jsonWorker();
    return new editorWorker();
  },
};

// Use our bundled monaco instance instead of the default CDN loader.
loader.config({ monaco });

export { monaco };
