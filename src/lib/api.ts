// Thin wrappers over the Tauri command layer (src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { RuleDoc, ValidationReport } from "../types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Open a URL in the OS browser (Tauri) or a new tab (browser dev fallback). */
export async function openExternal(url: string): Promise<void> {
  if (isTauri()) {
    try {
      await openUrl(url);
      return;
    } catch {
      /* fall through */
    }
  }
  window.open(url, "_blank", "noreferrer");
}

export async function validateDocument(
  content: string,
  targetDmsVersion: string,
): Promise<ValidationReport> {
  return invoke<ValidationReport>("validate", { content, targetDmsVersion });
}

export async function listRules(): Promise<RuleDoc[]> {
  return invoke<RuleDoc[]>("list_rules");
}

export async function readTextFile(path: string): Promise<string> {
  return invoke<string>("read_text_file", { path });
}

export async function writeTextFile(path: string, content: string): Promise<void> {
  await invoke("write_text_file", { path, content });
}
