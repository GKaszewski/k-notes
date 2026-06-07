import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { addToMutationQueue } from "@/lib/db";
import { toast } from "sonner";

export interface Note {
    id: string;
    title: string | null;
    content: string;
    is_pinned: boolean;
    is_archived: boolean;
    color: string;
    tags: Tag[];
    created_at: string;
    updated_at: string;
}

export interface Tag {
    id: string;
    name: string;
}

export interface CreateNoteInput {
    title?: string;
    content: string;
    tags?: string[];
    color?: string;
    is_pinned?: boolean;
}

export interface UpdateNoteInput {
    id: string;
    title?: string;
    content?: string;
    tags?: string[];
    color?: string;
    is_pinned?: boolean;
    is_archived?: boolean;
}

export function useNotes(params?: { pinned?: boolean; archived?: boolean; tag?: string }) {
    const searchParams = new URLSearchParams();
    if (params?.pinned !== undefined) searchParams.set("pinned", String(params.pinned));
    if (params?.archived !== undefined) searchParams.set("archived", String(params.archived));
    if (params?.tag) searchParams.set("tag", params.tag);

    return useQuery({
        queryKey: ["notes", params],
        queryFn: () => api.get(`/notes?${searchParams.toString()}`),
    });
}

export function useSearchNotes(query: string) {
    return useQuery({
        queryKey: ["notes", "search", query],
        queryFn: () => api.get(`/search?q=${encodeURIComponent(query)}`),
        enabled: query.length > 0,
    });
}

export function useCreateNote() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async (data: CreateNoteInput) => {
            const { tags, ...noteData } = data;

            const queueOffline = async () => {
                await addToMutationQueue({ type: "POST", endpoint: "/notes", body: data });
                toast.info("Note created offline. Will sync when online.");
                return {
                    id: crypto.randomUUID(),
                    ...noteData,
                    title: noteData.title ?? null,
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                    is_pinned: data.is_pinned || false,
                    is_archived: false,
                    tags: [],
                    color: data.color || "DEFAULT",
                };
            };

            if (!navigator.onLine) return queueOffline();

            try {
                const note = await api.post("/notes", noteData);

                // Tags are added separately via the dedicated endpoint.
                if (tags && tags.length > 0) {
                    for (const tag_name of tags) {
                        await api.post(`/notes/${note.id}/tags`, { tag_name });
                    }
                }

                return note;
            } catch (error: any) {
                if (!navigator.onLine || error.name === "AbortError" || error instanceof TypeError) {
                    return queueOffline();
                }
                throw error;
            }
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
            queryClient.invalidateQueries({ queryKey: ["tags"] });
        },
    });
}

export function useUpdateNote() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async ({ id, is_pinned, is_archived, tags, ...noteData }: UpdateNoteInput) => {
            const queueOffline = async () => {
                await addToMutationQueue({ type: "PATCH", endpoint: `/notes/${id}`, body: noteData });
                toast.info("Note updated offline. Will sync when online.");
                return { id, ...noteData };
            };

            if (!navigator.onLine) return queueOffline();

            try {
                // Core fields: title, content, color.
                if (Object.keys(noteData).length > 0) {
                    await api.patch(`/notes/${id}`, noteData);
                }

                // Pin / archive via dedicated endpoints.
                if (is_pinned !== undefined) {
                    await api.patch(`/notes/${id}/pin`, { pinned: is_pinned });
                }
                if (is_archived !== undefined) {
                    await api.patch(`/notes/${id}/archive`, { archived: is_archived });
                }

                // Tag changes: diff current tags (from cache or fresh fetch) against desired list.
                if (tags !== undefined) {
                    const allNotes = queryClient.getQueriesData<Note[]>({ queryKey: ["notes"] });
                    let currentNote: Note | undefined;
                    for (const [, notes] of allNotes) {
                        currentNote = (notes as Note[])?.find(n => n.id === id);
                        if (currentNote) break;
                    }

                    // Fallback: fetch from server if not in cache (or cache was corrupted).
                    if (!currentNote || currentNote.tags.some(t => !t.id)) {
                        currentNote = await api.get(`/notes/${id}`);
                    }

                    if (currentNote) {
                        for (const existing of currentNote.tags) {
                            if (!tags.includes(existing.name)) {
                                await api.delete(`/notes/${id}/tags/${existing.id}`);
                            }
                        }
                        for (const tag_name of tags) {
                            if (!currentNote.tags.some(t => t.name === tag_name)) {
                                await api.post(`/notes/${id}/tags`, { tag_name });
                            }
                        }
                    }
                }

                return { id };
            } catch (error: any) {
                if (!navigator.onLine || error.name === "AbortError" || error instanceof TypeError) {
                    return queueOffline();
                }
                throw error;
            }
        },

        onMutate: async (updatedNote) => {
            await queryClient.cancelQueries({ queryKey: ["notes"] });
            const previousNotes = queryClient.getQueriesData({ queryKey: ["notes"] });

            // Exclude tags from the optimistic update: tags are string[] in UpdateNoteInput
            // but Tag[] in the cache. Spreading strings would corrupt the cached Tag objects
            // (losing .id) which the mutationFn reads for the add/remove diff.
            const { tags: _tags, ...optimisticData } = updatedNote;

            queryClient.setQueriesData({ queryKey: ["notes"] }, (old: Note[] | undefined) => {
                if (!old) return old;
                return old.map(note =>
                    note.id === updatedNote.id ? { ...note, ...optimisticData } : note
                );
            });

            return { previousNotes };
        },

        onError: (_err, _updatedNote, context) => {
            if (context?.previousNotes) {
                context.previousNotes.forEach(([queryKey, data]) => {
                    queryClient.setQueryData(queryKey, data);
                });
            }
        },

        onSettled: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
            queryClient.invalidateQueries({ queryKey: ["tags"] });
        },
    });
}

export function useDeleteNote() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async (id: string) => {
            const queueOffline = async () => {
                await addToMutationQueue({ type: "DELETE", endpoint: `/notes/${id}` });
                toast.info("Note deleted offline. Will sync when online.");
                return { id };
            };

            if (!navigator.onLine) return queueOffline();

            try {
                return await api.delete(`/notes/${id}`);
            } catch (error: any) {
                if (!navigator.onLine || error.name === "AbortError" || error instanceof TypeError) {
                    return queueOffline();
                }
                throw error;
            }
        },

        onMutate: async (deletedId) => {
            await queryClient.cancelQueries({ queryKey: ["notes"] });
            const previousNotes = queryClient.getQueriesData({ queryKey: ["notes"] });

            queryClient.setQueriesData({ queryKey: ["notes"] }, (old: Note[] | undefined) => {
                if (!old) return old;
                return old.filter(note => note.id !== deletedId);
            });

            return { previousNotes };
        },

        onError: (_err, _deletedId, context) => {
            if (context?.previousNotes) {
                context.previousNotes.forEach(([queryKey, data]) => {
                    queryClient.setQueryData(queryKey, data);
                });
            }
        },

        onSettled: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
            queryClient.invalidateQueries({ queryKey: ["tags"] });
        },
    });
}

export interface NoteVersion {
    id: string;
    note_id: string;
    title: string | null;
    content: string;
    created_at: string;
}

export function useNoteVersions(noteId: string, enabled: boolean = false) {
    return useQuery({
        queryKey: ["notes", noteId, "versions"],
        queryFn: () => api.get(`/notes/${noteId}/versions`),
        enabled: enabled && !!noteId,
    });
}

export function useTags() {
    return useQuery({
        queryKey: ["tags"],
        queryFn: () => api.get("/tags"),
    });
}

export function useDeleteTag() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: (id: string) => api.delete(`/tags/${id}`),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["tags"] });
            queryClient.invalidateQueries({ queryKey: ["notes"] });
        },
    });
}

export function useRenameTag() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ id, name }: { id: string; name: string }) =>
            api.patch(`/tags/${id}`, { name }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["tags"] });
            queryClient.invalidateQueries({ queryKey: ["notes"] });
        },
    });
}
