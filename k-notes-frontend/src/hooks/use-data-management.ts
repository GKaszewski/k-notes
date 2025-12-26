import { useRef } from "react";
import { api } from "@/lib/api";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

export function useDataManagement() {
    const { t } = useTranslation();
    const fileInputRef = useRef<HTMLInputElement>(null);

    const exportData = async () => {
        try {
            const blob = await api.exportData();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `k-notes-backup-${new Date().toISOString().split('T')[0]}.json`;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
            toast.success(t("Export successful"));
        } catch (e) {
            toast.error(t("Export failed"));
        }
    };

    const importData = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        try {
            const text = await file.text();
            const data = JSON.parse(text);
            await api.importData(data);
            toast.success(t("Import successful. Reloading..."));
            setTimeout(() => window.location.reload(), 1000);
        } catch (e) {
            console.error(e);
            toast.error(t("Import failed"));
        }
    };

    const triggerImport = () => {
        fileInputRef.current?.click();
    };

    return {
        fileInputRef,
        exportData,
        importData,
        triggerImport,
    };
}
