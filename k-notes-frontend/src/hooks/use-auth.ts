import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useNavigate } from "react-router-dom";

export interface User {
    id: string;
    email: string;
    created_at: string;
}

// Fetch current user
async function fetchUser(): Promise<User | null> {
    try {
        const user = await api.get("/auth/me");
        return user;
    } catch (error: any) {
        if (error.status === 401) {
            return null; // Not logged in
        }
        throw error;
    }
}

export function useUser() {
    return useQuery({
        queryKey: ["user"],
        queryFn: fetchUser,
        retry: false, // Don't retry on 401
        staleTime: 1000 * 60 * 5, // 5 minutes
    });
}

export function useLogin() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: (credentials: any) => api.post("/auth/login", credentials),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["user"] });
            navigate("/");
        },
    });
}

export function useRegister() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: (credentials: any) => api.post("/auth/register", credentials),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["user"] });
            navigate("/");
        },
    });
}

export function useLogout() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: () => api.post("/auth/logout", {}),
        onSuccess: () => {
            queryClient.setQueryData(["user"], null);
            navigate("/login");
        },
    });
}
