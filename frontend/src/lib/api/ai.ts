import {
  cmdGetAiSettings,
  cmdSaveAiSettings,
  cmdSetAiApiKey,
  cmdClearAiApiKey,
  cmdAiPreviewContext,
  cmdAiChat,
  type AiSettings,
  type AiChatResponse,
  type ContextPayload,
  type PrivacyLevel
} from '$lib/generated/commands';

export type { AiSettings, AiChatResponse, ContextPayload, PrivacyLevel };

export async function getAiSettings(): Promise<AiSettings> {
  return await cmdGetAiSettings();
}

export async function saveAiSettings(settings: AiSettings): Promise<void> {
  await cmdSaveAiSettings({ settings });
}

export async function setAiApiKey(apiKey: string): Promise<void> {
  await cmdSetAiApiKey({ apiKey });
}

export async function clearAiApiKey(): Promise<void> {
  await cmdClearAiApiKey();
}

export async function previewAiContext(level: PrivacyLevel): Promise<ContextPayload> {
  return await cmdAiPreviewContext({ level });
}

export async function chatWithAi(
  prompt: string,
  privacyLevel?: PrivacyLevel | null,
  consentGiven?: boolean | null
): Promise<AiChatResponse> {
  return await cmdAiChat({
    prompt,
    privacyLevel: privacyLevel ?? undefined,
    consentGiven: consentGiven ?? undefined
  });
}
