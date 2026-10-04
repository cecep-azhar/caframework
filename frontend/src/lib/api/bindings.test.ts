import { describe, it, expect, vi } from 'vitest';
import * as vaultApi from './vault';
import * as aiApi from './ai';
import * as notesApi from './notes';
import * as profilesApi from './profiles';

// Mock @tauri-apps/api/core
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args)
}));

describe('Frontend API Bindings Unit Tests (F7.5)', () => {
  it('vault: isVaultInitialized invokes is_vault_initialized', async () => {
    mockInvoke.mockResolvedValueOnce(true);
    const res = await vaultApi.isVaultInitialized();
    expect(res).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('is_vault_initialized');
  });

  it('vault: changeMasterPassword forwards camelCase args', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await vaultApi.changeMasterPassword('old_pass', 'new_pass');
    expect(mockInvoke).toHaveBeenCalledWith('change_master_password', {
      oldPassword: 'old_pass',
      newPassword: 'new_pass'
    });
  });

  it('ai: getAiSettings & saveAiSettings work through typed IPC bindings', async () => {
    const fakeSettings: any = {
      enabled: true,
      provider: 'openai',
      api_key: 'sk-test',
      model: 'gpt-4o',
      endpoint: null
    };
    mockInvoke.mockResolvedValueOnce(fakeSettings);
    const res = await aiApi.getAiSettings();
    expect(res).toEqual(fakeSettings);
    expect(mockInvoke).toHaveBeenCalledWith('get_ai_settings');

    mockInvoke.mockResolvedValueOnce(undefined);
    await aiApi.saveAiSettings(fakeSettings);
    expect(mockInvoke).toHaveBeenCalledWith('save_ai_settings', { settings: fakeSettings });
  });

  it('notes: listNotes, saveNote, deleteNote work through typed IPC bindings', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    await notesApi.listNotes('prof_1', true);
    expect(mockInvoke).toHaveBeenCalledWith('list_notes', {
      callerProfileId: 'prof_1',
      isOwner: true
    });

    const noteInput = {
      title: 'Hello',
      content: 'World',
      visibility: 'shared'
    };
    mockInvoke.mockResolvedValueOnce({ id: '1', ...noteInput });
    await notesApi.saveNote(noteInput, 'prof_1');
    expect(mockInvoke).toHaveBeenCalledWith('save_note', {
      input: noteInput,
      callerProfileId: 'prof_1'
    });

    mockInvoke.mockResolvedValueOnce(undefined);
    await notesApi.deleteNote('1', 'prof_1');
    expect(mockInvoke).toHaveBeenCalledWith('delete_note', {
      id: '1',
      callerProfileId: 'prof_1'
    });
  });

  it('profiles: listProfiles & verifyPin work through typed IPC bindings', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    await profilesApi.listProfiles();
    expect(mockInvoke).toHaveBeenCalledWith('list_profiles');

    mockInvoke.mockResolvedValueOnce(true);
    const verified = await profilesApi.verifyPin('prof_1', '1234');
    expect(verified).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('verify_pin', {
      profileId: 'prof_1',
      pin: '1234'
    });
  });
});
