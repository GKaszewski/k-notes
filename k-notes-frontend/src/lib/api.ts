const getApiUrl = () => {
    const stored = localStorage.getItem("k_notes_api_url");
    return stored ? `${stored}/api/v1` : "http://localhost:3000/api/v1";
};

export const getBaseUrl = () => {
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

    const headers = {
        "Content-Type": "application/json",
        ...options.headers,
    };

    const config: RequestInit = {
        ...options,
        headers,
        credentials: "include", // Important for cookies!
    };

    const response = await fetch(url, config);

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
};
