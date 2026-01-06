import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, setAuthToken, clearAuthToken, getBaseUrl } from "@/lib/api";
import { useNavigate } from "react-router-dom";

export interface User {
    id: string;
    email: string;
    created_at: string;
}

// Token response from JWT/OIDC login
export interface TokenResponse {
    access_token: string;
    token_type: string;
    expires_in: number;
}

// Login can return either User (session mode) or Token (JWT mode)
export type LoginResult = User | TokenResponse;

function isTokenResponse(result: LoginResult): result is TokenResponse {
    return 'access_token' in result;
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
        mutationFn: (credentials: { email: string; password: string }): Promise<LoginResult> =>
            api.post("/auth/login", credentials),
        onSuccess: (result: LoginResult) => {
            // If we got a token response, store the token
            if (isTokenResponse(result)) {
                setAuthToken(result.access_token);
            }
            queryClient.invalidateQueries({ queryKey: ["user"] });
            navigate("/");
        },
    });
}

export function useRegister() {
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    return useMutation({
        mutationFn: (credentials: { email: string; password: string }): Promise<LoginResult> =>
            api.post("/auth/register", credentials),
        onSuccess: (result: LoginResult) => {
            // If we got a token response, store the token
            if (isTokenResponse(result)) {
                setAuthToken(result.access_token);
            }
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
            // Clear both session data and JWT token
            clearAuthToken();
            queryClient.setQueryData(["user"], null);
            navigate("/login");
        },
        onError: () => {
            // Even on error, clear local state
            clearAuthToken();
            queryClient.setQueryData(["user"], null);
            navigate("/login");
        },
    });
}

// Hook to initiate OIDC login flow
export function useOidcLogin() {
    return () => {
        // Redirect to OIDC login endpoint
        window.location.href = `${getBaseUrl()}/api/v1/auth/login/oidc`;
    };
}

