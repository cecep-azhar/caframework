import { invoke } from '@tauri-apps/api/core';

export interface AiSettings {
  provider: 'openai' | 'anthropic' | 'ollama' | 'hosted';
  endpoint: string;
  api_key: string;
  model: string;
  privacy_mode: boolean;
  guardrails_enabled: boolean;
}

export interface AiChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export async function getAiSettings(): Promise<AiSettings> {
  return await invoke<AiSettings>('get_ai_settings');
}

export async function saveAiSettings(settings: AiSettings): Promise<void> {
  await invoke('save_ai_settings', { settings });
}

export async function chatWithAi(prompt: string, context?: string): Promise<string> {
  return await invoke<string>('ai_chat', { prompt, context });
}
