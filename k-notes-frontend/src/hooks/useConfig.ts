
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export type AuthMode = 'session' | 'jwt' | 'both';

export interface ConfigResponse {
    allow_registration: boolean;
    auth_mode: AuthMode;
    oidc_enabled: boolean;
    password_login_enabled: boolean;
}

export function useConfig() {
    return useQuery<ConfigResponse>({
        queryKey: ["config"],
        queryFn: () => api.get("/config"),
        staleTime: Infinity, // Config rarely changes
    });
}

