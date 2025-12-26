import { useState, useEffect } from "react";
import { getBaseUrl } from "@/lib/api";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

export function useApiUrl() {
    const { t } = useTranslation();
    const [apiUrl, setApiUrl] = useState("");
    const [currentApiUrl, setCurrentApiUrl] = useState("");

    useEffect(() => {
        const url = getBaseUrl();
        setCurrentApiUrl(url);
        setApiUrl(localStorage.getItem("k_notes_api_url") || "");
    }, []);

    const saveApiUrl = (url: string) => {
        const trimmedUrl = url.trim();

        if (!trimmedUrl) {
            toast.error(t("Please enter a URL"));
            return false;
        }

        try {
            new URL(trimmedUrl);
            const cleanUrl = trimmedUrl.replace(/\/$/, "");
            localStorage.setItem("k_notes_api_url", cleanUrl);
            toast.success(t("API URL updated successfully. Please reload the page."), {
                action: {
                    label: t("Reload"),
                    onClick: () => window.location.reload(),
                },
            });
            setCurrentApiUrl(cleanUrl);
            return true;
        } catch {
            toast.error(t("Invalid URL format. Please enter a valid URL."));
            return false;
        }
    };

    const resetApiUrl = () => {
        localStorage.removeItem("k_notes_api_url");
        setApiUrl("");
        const defaultUrl = window.env?.API_URL || "http://localhost:3000";
        setCurrentApiUrl(defaultUrl);
        toast.success(t("API URL reset to default. Please reload the page."), {
            action: {
                label: t("Reload"),
                onClick: () => window.location.reload(),
            },
        });
    };

    return {
        apiUrl,
        setApiUrl,
        currentApiUrl,
        saveApiUrl,
        resetApiUrl,
    };
}
