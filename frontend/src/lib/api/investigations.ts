import { invoke } from '@tauri-apps/api/core';

export interface InvestigationRecord {
  id: string;
  title: string;
  host_id: string | null;
  status: string;
  notes: string;
  evidence: string;
  created_at: number;
  updated_at: number;
}

export interface InvestigationInput {
  id?: string;
  title: string;
  host_id: string | null;
  status: string;
  notes: string;
  evidence: string;
}

export async function listInvestigations(): Promise<InvestigationRecord[]> {
  return invoke('list_investigations');
}

export async function saveInvestigation(input: InvestigationInput): Promise<InvestigationRecord> {
  return invoke('save_investigation', { input });
}

export async function deleteInvestigation(id: string): Promise<void> {
  return invoke('delete_investigation', { id });
}
