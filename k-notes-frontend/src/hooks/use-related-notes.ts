import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface NoteLink {
    source_note_id: string;
    target_note_id: string;
    score: number;
    created_at: string;
}

export function useRelatedNotes(noteId: string | undefined) {
    const { data, error, isLoading } = useQuery({
        queryKey: ["notes", noteId, "related"],
        queryFn: () => api.get(`/notes/${noteId}/related`),
        enabled: !!noteId,
    });

    return {
        relatedLinks: data as NoteLink[] | undefined,
        isRelatedLoading: isLoading,
        relatedError: error,
    };
}
