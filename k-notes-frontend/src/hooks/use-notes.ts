import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { addToMutationQueue } from "@/lib/db";
import { toast } from "sonner";

export interface Note {
    id: string;
    title: string;
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
    created_at?: string;
}

export interface CreateNoteInput {
    title: string;
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
    // Construct query string
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
            const queueOffline = async () => {
                console.log("Queueing offline creation...");
                await addToMutationQueue({
                    type: "POST",
                    endpoint: "/notes",
                    body: data,
                });
                console.log("Offline creation queued.");
                toast.info("Note created offline. Will sync when online.");
                return {
                    id: crypto.randomUUID(),
                    ...data,
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                    is_pinned: data.is_pinned || false,
                    is_archived: false,
                    tags: [],
                    color: "default"
                };
            };

            if (!navigator.onLine) {
                console.log("Navigator is offline, queueing");
                return queueOffline();
            }

            try {
                return await api.post("/notes", data);
            } catch (error: any) {
                console.error("API Error in createNote:", error);
                if (!navigator.onLine || error.name === 'AbortError' || error instanceof TypeError) {
                    console.log("Falling back to offline queue due to error");
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
        mutationFn: async ({ id, ...data }: UpdateNoteInput) => {
            const queueOffline = async () => {
                await addToMutationQueue({
                    type: "PATCH",
                    endpoint: `/notes/${id}`,
                    body: data,
                });
                toast.info("Note updated offline. Will sync when online.");
                return { id, ...data };
            };

            if (!navigator.onLine) {
                return queueOffline();
            }

            try {
                return await api.patch(`/notes/${id}`, data);
            } catch (error: any) {
                if (!navigator.onLine || error.name === 'AbortError' || error instanceof TypeError) {
                    return queueOffline();
                }
                throw error;
            }
        },

        // Optimistic update
        onMutate: async (updatedNote) => {
            // Cancel any outgoing refetches
            await queryClient.cancelQueries({ queryKey: ["notes"] });

            // Snapshot the previous value
            const previousNotes = queryClient.getQueriesData({ queryKey: ["notes"] });

            // Optimistically update all matching queries
            queryClient.setQueriesData({ queryKey: ["notes"] }, (old: Note[] | undefined) => {
                if (!old) return old;
                return old.map((note) =>
                    note.id === updatedNote.id
                        ? { ...note, ...updatedNote }
                        : note
                );
            });

            // Return a context object with the snapshotted value
            return { previousNotes };
        },

        // If the mutation fails, use the context returned from onMutate to roll back
        onError: (_err, _updatedNote, context) => {
            if (context?.previousNotes) {
                context.previousNotes.forEach(([queryKey, data]) => {
                    queryClient.setQueryData(queryKey, data);
                });
            }
        },

        // Always refetch after error or success
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
                await addToMutationQueue({
                    type: "DELETE",
                    endpoint: `/notes/${id}`,
                });
                toast.info("Note deleted offline. Will sync when online.");
                return { id };
            };

            if (!navigator.onLine) {
                return queueOffline();
            }

            try {
                return await api.delete(`/notes/${id}`);
            } catch (error: any) {
                if (!navigator.onLine || error.name === 'AbortError' || error instanceof TypeError) {
                    return queueOffline();
                }
                throw error;
            }
        },

        // Optimistic delete
        onMutate: async (deletedId) => {
            // Cancel any outgoing refetches
            await queryClient.cancelQueries({ queryKey: ["notes"] });

            // Snapshot the previous value
            const previousNotes = queryClient.getQueriesData({ queryKey: ["notes"] });

            // Optimistically remove from all matching queries
            queryClient.setQueriesData({ queryKey: ["notes"] }, (old: Note[] | undefined) => {
                if (!old) return old;
                return old.filter((note) => note.id !== deletedId);
            });

            // Return a context object with the snapshotted value
            return { previousNotes };
        },

        // If the mutation fails, use the context returned from onMutate to roll back
        onError: (_err, _deletedId, context) => {
            if (context?.previousNotes) {
                context.previousNotes.forEach(([queryKey, data]) => {
                    queryClient.setQueryData(queryKey, data);
                });
            }
        },

        // Always refetch after error or success
        onSettled: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
            queryClient.invalidateQueries({ queryKey: ["tags"] });
        },
    });
}

export interface NoteVersion {
    id: string;
    note_id: string;
    title: string;
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
