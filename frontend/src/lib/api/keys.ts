import { invoke } from '@tauri-apps/api/core';

export interface KeyRecord {
  id: string;
  name: string;
  algorithm: string;
  fingerprint: string;
  publicKey: string;
  createdAt: string;
}

export interface KeyInput {
  name: string;
  algorithm: string; // "Ed25519" | "RSA-4096"
}

export function listKeys(): Promise<KeyRecord[]> {
  return invoke('list_keys');
}

export function generateKey(input: KeyInput): Promise<KeyRecord> {
  return invoke('generate_key', { input });
}

export function importKey(name: string, privateKeyPem: string, passphrase?: string): Promise<KeyRecord> {
  return invoke('import_key', { name, privateKeyPem, passphrase });
}

export function deleteKey(id: string): Promise<void> {
  return invoke('delete_key', { id });
}

export function deployPublicKey(hostId: string, keyId: string): Promise<void> {
  return invoke('deploy_public_key', { hostId, keyId });
}