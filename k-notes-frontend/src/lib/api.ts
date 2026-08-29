declare global {
    interface Window {
        env?: {
            API_URL?: string;
        };
    }
}

const ACCESS_TOKEN_KEY = 'k_notes_auth_token';
const REFRESH_TOKEN_KEY = 'k_notes_refresh_token';

export function setAuthToken(token: string): void {
    localStorage.setItem(ACCESS_TOKEN_KEY, token);
}

export function getAuthToken(): string | null {
    return localStorage.getItem(ACCESS_TOKEN_KEY);
}

export function clearAuthToken(): void {
    localStorage.removeItem(ACCESS_TOKEN_KEY);
}

export function setRefreshToken(token: string): void {
    localStorage.setItem(REFRESH_TOKEN_KEY, token);
}

export function getRefreshToken(): string | null {
    return localStorage.getItem(REFRESH_TOKEN_KEY);
}

export function clearRefreshToken(): void {
    localStorage.removeItem(REFRESH_TOKEN_KEY);
}

export function clearAllTokens(): void {
    clearAuthToken();
    clearRefreshToken();
}

const getApiUrl = () => {
    // 1. Runtime config injected via window.env (e.g. for cross-origin setups)
    if (window.env?.API_URL) {
        return `${window.env.API_URL}/api/v1`;
    }
    // 2. LocalStorage override (user-configurable in settings)
    const stored = localStorage.getItem("k_notes_api_url");
    if (stored) {
        return `${stored}/api/v1`;
    }
    // 3. Same-origin fallback — works when SPA is served by the backend process
    return "/api/v1";
};

export const getBaseUrl = () => {
    if (window.env?.API_URL) {
        return window.env.API_URL;
    }
    const stored = localStorage.getItem("k_notes_api_url");
    return stored ? stored : "http://localhost:3000";
}

export class ApiError extends Error {
    public status: number;

    constructor(status: number, message: string) {
        super(message);
        this.status = status;
        this.name = "ApiError";
    }
}

let refreshPromise: Promise<boolean> | null = null;

async function tryRefreshToken(): Promise<boolean> {
    const refreshToken = getRefreshToken();
    if (!refreshToken) return false;

    try {
        const url = `${getApiUrl()}/auth/refresh`;
        const response = await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ refresh_token: refreshToken }),
        });

        if (!response.ok) return false;

        const data = await response.json();
        setAuthToken(data.access_token);
        setRefreshToken(data.refresh_token);
        return true;
    } catch {
        return false;
    }
}

async function fetchWithAuth(endpoint: string, options: RequestInit = {}) {
    const url = `${getApiUrl()}${endpoint}`;
    const token = getAuthToken();

    const headers: Record<string, string> = {
        "Content-Type": "application/json",
        ...(options.headers as Record<string, string> || {}),
    };

    // Add Authorization header if we have a JWT token
    if (token) {
        headers["Authorization"] = `Bearer ${token}`;
    }

    const config: RequestInit = {
        ...options,
        headers,
        credentials: "include", // Still include for session-based auth
    };

    try {
        const fetchPromise = fetch(url, config);
        const timeoutPromise = new Promise((_, reject) =>
            setTimeout(() => reject(new TypeError("Network request timed out")), 3000)
        );

        const response = (await Promise.race([fetchPromise, timeoutPromise])) as Response;

        if (response.status === 401) {
            if (!refreshPromise) {
                refreshPromise = tryRefreshToken().finally(() => {
                    refreshPromise = null;
                });
            }

            const refreshed = await refreshPromise;
            if (refreshed) {
                // Retry with the new token
                const newToken = getAuthToken();
                if (newToken) {
                    headers["Authorization"] = `Bearer ${newToken}`;
                }
                const retryResponse = await fetch(url, { ...options, headers, credentials: "include" });

                if (!retryResponse.ok) {
                    let errorMessage = "An error occurred";
                    try {
                        const errorData = await retryResponse.json();
                        errorMessage = errorData.error?.message || errorData.message || errorMessage;
                    } catch {
                        // failed to parse json
                    }
                    throw new ApiError(retryResponse.status, errorMessage);
                }

                if (retryResponse.status === 204) return null;
                try {
                    return await retryResponse.json();
                } catch {
                    return null;
                }
            }

            // Refresh failed — clear tokens and redirect
            clearAllTokens();
            window.location.href = "/login";
            throw new ApiError(401, "Session expired");
        }

        if (!response.ok) {
            // Try to parse error message
            let errorMessage = "An error occurred";
            try {
                const errorData = await response.json();
                errorMessage = errorData.error?.message || errorData.message || errorMessage;
            } catch {
                // failed to parse json
            }

            throw new ApiError(response.status, errorMessage);
        }

        // For 204 No Content or empty responses
        if (response.status === 204) {
            return null;
        }

        // Try to parse JSON
        try {
            return await response.json();
        } catch {
            return null;
        }
    } catch (error) {

        throw error;
    }
}



export const api = {
    get: (endpoint: string) => fetchWithAuth(endpoint, { method: "GET" }),
    post: (endpoint: string, body: any) =>
        fetchWithAuth(endpoint, {
            method: "POST",
            body: JSON.stringify(body),
        }),
    patch: (endpoint: string, body: any) =>
        fetchWithAuth(endpoint, {
            method: "PATCH",
            body: JSON.stringify(body),
        }),
    delete: (endpoint: string) => fetchWithAuth(endpoint, { method: "DELETE" }),
    exportData: async () => {
        const token = getAuthToken();
        const headers: Record<string, string> = {};
        if (token) {
            headers["Authorization"] = `Bearer ${token}`;
        }
        const response = await fetch(`${getApiUrl()}/export`, {
            credentials: "include",
            headers,
        });
        if (!response.ok) throw new ApiError(response.status, "Failed to export data");
        return response.blob();
    },
    importData: (data: any) => api.post("/import", data),
};
