import {
  cmdListNotes,
  cmdSaveNote,
  cmdDeleteNote,
  type NoteRecord,
  type NoteInput
} from '$lib/generated/commands';

export type { NoteRecord, NoteInput };

export async function listNotes(callerProfileId: string, isOwner: boolean): Promise<NoteRecord[]> {
  return await cmdListNotes({ callerProfileId, isOwner });
}

export async function saveNote(input: NoteInput, callerProfileId: string): Promise<NoteRecord> {
  return await cmdSaveNote({ input, callerProfileId });
}

export async function deleteNote(id: string, callerProfileId: string): Promise<void> {
  await cmdDeleteNote({ id, callerProfileId });
}
