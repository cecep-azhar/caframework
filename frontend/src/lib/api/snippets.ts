// JS <-> Rust binding for snippet persistence. Mirrors `caf_core::snippets` 1:1 —
// see crates/caf-core/src/snippets.rs for the source of truth on shapes and error codes.
import { invoke } from '@tauri-apps/api/core';

export interface SnippetRecord {
  id: string;
  label: string;
  description: string;
  command: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
}

export interface SnippetInput {
  id?: string;
  label: string;
  description: string;
  command: string;
  tags: string[];
}

export function listSnippets(): Promise<SnippetRecord[]> {
  return invoke('list_snippets');
}

export function saveSnippet(input: SnippetInput): Promise<SnippetRecord> {
  return invoke('save_snippet', { input });
}

export function deleteSnippet(id: string): Promise<void> {
  return invoke('delete_snippet', { id });
}
