import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, setAuthToken, setRefreshToken, clearAllTokens, getRefreshToken } from "@/lib/api";
import { useNavigate } from "react-router-dom";

export interface User {
    id: string;
    email: string;
    created_at: string;
}

export interface AuthResponse {
    user: User;
    access_token: string;
    refresh_token: string;
}

async function fetchUser(): Promise<User | null> {
    try {
        return await api.get("/auth/me");
    } catch (error: any) {
        if (error.status === 401) return null;
        throw error;
    }
}

export function useUser() {
    return useQuery({
        queryKey: ["user"],
        queryFn: fetchUser,
        retry: false,
        staleTime: 1000 * 60 * 5,
    });
}

export function useLogin() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: (credentials: { email: string; password: string }): Promise<AuthResponse> =>
            api.post("/auth/login", credentials),
        onSuccess: (result: AuthResponse) => {
            setAuthToken(result.access_token);
            setRefreshToken(result.refresh_token);
            queryClient.invalidateQueries({ queryKey: ["user"] });
            navigate("/");
        },
    });
}

export function useRegister() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: (credentials: { email: string; password: string }): Promise<AuthResponse> =>
            api.post("/auth/register", credentials),
        onSuccess: (result: AuthResponse) => {
            setAuthToken(result.access_token);
            setRefreshToken(result.refresh_token);
            queryClient.invalidateQueries({ queryKey: ["user"] });
            navigate("/");
        },
    });
}

export function useLogout() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: async () => {
            const refreshToken = getRefreshToken();
            if (refreshToken) {
                await api.post("/auth/logout", { refresh_token: refreshToken }).catch(() => {});
            }
        },
        onSettled: () => {
            clearAllTokens();
            queryClient.setQueryData(["user"], null);
            navigate("/login");
        },
    });
}
