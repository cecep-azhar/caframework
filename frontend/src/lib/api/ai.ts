import { invoke } from '@tauri-apps/api/core';

export interface AiPlanStep {
  id?: string;
  step_number: number;
  title: string;
  command: string;
  description: string;
  is_sudo?: boolean;
  is_danger?: boolean;
  action_type?: string;
  action_name?: string;
  action_params?: any;
}

export interface AiSettings {
  provider: string;
  base_url: string;
  api_key: string;
  model: string;
}

export interface AiChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface AiChatReply {
  reply: string;
  ready: boolean;
  steps: AiPlanStep[];
}

export async function getAiSettings(): Promise<AiSettings> {
  return invoke<AiSettings>('get_ai_settings');
}

export async function saveAiSettings(settings: AiSettings): Promise<void> {
  return invoke<void>('save_ai_settings', { settings });
}

export async function aiChat(
  messages: AiChatMessage[],
  hostLabel?: string,
  hosted: boolean = false
): Promise<AiChatReply> {
  return invoke<AiChatReply>('ai_chat', {
    messages,
    hostLabel,
    hosted
  });
}
