declare global {
    interface Window {
        env?: {
            API_URL?: string;
        };
    }
}

const TOKEN_STORAGE_KEY = 'k_notes_auth_token';

// JWT Token management
export function setAuthToken(token: string): void {
    localStorage.setItem(TOKEN_STORAGE_KEY, token);
}

export function getAuthToken(): string | null {
    return localStorage.getItem(TOKEN_STORAGE_KEY);
}

export function clearAuthToken(): void {
    localStorage.removeItem(TOKEN_STORAGE_KEY);
}

const getApiUrl = () => {
    // 1. Runtime config (Docker)
    if (window.env?.API_URL) {
        return `${window.env.API_URL}/api/v1`;
    }
    // 2. LocalStorage override
    const stored = localStorage.getItem("k_notes_api_url");
    if (stored) {
        return `${stored}/api/v1`;
    }
    // 3. Default fallback
    return "http://localhost:3000/api/v1";
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

