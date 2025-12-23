import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

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
        mutationFn: (data: CreateNoteInput) => api.post("/notes", data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
        },
    });
}

export function useUpdateNote() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ id, ...data }: UpdateNoteInput) => api.patch(`/notes/${id}`, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
        },
    });
}

export function useDeleteNote() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: (id: string) => api.delete(`/notes/${id}`),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["notes"] });
        },
    });
}

export function useTags() {
    return useQuery({
        queryKey: ["tags"],
        queryFn: () => api.get("/tags"),
    });
}
