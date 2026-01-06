import { useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { setAuthToken } from "@/lib/api";
import { useTranslation } from "react-i18next";

/**
 * OIDC Callback Handler
 *
 * This page handles redirects from the OIDC provider after authentication.
 *
 * In Session mode: The backend sets a session cookie during the callback,
 * so we just need to redirect to the dashboard.
 *
 * In JWT mode: The backend redirects here with a token in the URL fragment
 * or query params, which we need to extract and store.
 */
export default function OidcCallbackPage() {
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const queryClient = useQueryClient();
    const { t } = useTranslation();

    useEffect(() => {
        // Check for token in URL hash (implicit flow) or query params
        const hashParams = new URLSearchParams(window.location.hash.slice(1));
        const accessToken =
            hashParams.get("access_token") || searchParams.get("access_token");

        if (accessToken) {
            // JWT mode: store the token
            setAuthToken(accessToken);
        }

        // Invalidate user query to refetch with new auth state
        queryClient.invalidateQueries({ queryKey: ["user"] });

        // Redirect to dashboard
        navigate("/", { replace: true });
    }, [navigate, searchParams, queryClient]);

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-50 dark:bg-gray-950">
            <div className="text-center">
                <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mx-auto mb-4" />
                <p className="text-gray-500 dark:text-gray-400">
                    {t("Completing sign in...")}
                </p>
            </div>
        </div>
    );
}
