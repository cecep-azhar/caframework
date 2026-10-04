import {
  cmdGetAiSettings,
  cmdSaveAiSettings,
  cmdAiChat,
  type AiSettings,
  type AiChatResponse
} from '$lib/generated/commands';

export type { AiSettings, AiChatResponse };

export async function getAiSettings(): Promise<AiSettings> {
  return await cmdGetAiSettings();
}

export async function saveAiSettings(settings: AiSettings): Promise<void> {
  await cmdSaveAiSettings({ settings });
}

export async function chatWithAi(prompt: string, context?: string | null): Promise<AiChatResponse> {
  return await cmdAiChat({ prompt, context });
}
