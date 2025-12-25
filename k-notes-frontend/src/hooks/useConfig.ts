
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface ConfigResponse {
    allow_registration: boolean;
}

export function useConfig() {
    return useQuery<ConfigResponse>({
        queryKey: ["config"],
        queryFn: () => api.get("/config"),
        staleTime: Infinity, // Config rarely changes
    });
}
