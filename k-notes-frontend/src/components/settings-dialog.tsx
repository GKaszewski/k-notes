import { useRef, useState, useEffect } from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { Separator } from "@/components/ui/separator";
import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "@/components/language-switcher";

interface SettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    dataManagementEnabled: boolean;
}

export function SettingsDialog({ open, onOpenChange, dataManagementEnabled }: SettingsDialogProps) {
    const [url, setUrl] = useState("http://localhost:3000");
    const { t } = useTranslation();

    useEffect(() => {
        const stored = localStorage.getItem("k_notes_api_url");
        if (stored) {
            setUrl(stored);
        }
    }, [open]);

    const handleSave = () => {
        try {
            // Basic validation
            new URL(url);
            // Remove trailing slash if present
            const cleanUrl = url.replace(/\/$/, "");
            localStorage.setItem("k_notes_api_url", cleanUrl);
            toast.success(t("Settings saved. Please refresh the page."));
            onOpenChange(false);
            window.location.reload();
        } catch (e) {
            toast.error(t("Invalid URL"));
        }
    };

    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleExport = async () => {
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

    const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        try {
            const text = await file.text();
            const data = JSON.parse(text);
            await api.importData(data);
            toast.success(t("Import successful. Reloading..."));
            onOpenChange(false);
            window.location.reload();
        } catch (e) {
            console.error(e);
            toast.error(t("Import failed"));
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t("Settings")}</DialogTitle>
                    <DialogDescription>
                        {t("Configure the application settings.")}
                    </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Label htmlFor="url" className="text-right">
                            {t("Backend URL")}
                        </Label>
                        <Input
                            id="url"
                            value={url}
                            onChange={(e) => setUrl(e.target.value)}
                            className="col-span-3"
                            placeholder="http://localhost:3000"
                        />
                    </div>
                </div>

                <Separator className="my-2" />
                <LanguageSwitcher />

                {dataManagementEnabled && <>
                    <Separator className="my-2" />

                    <div className="py-4 space-y-4">
                        <div className="flex flex-col space-y-2">
                            <h4 className="font-medium leading-none">{t("Data Management")}</h4>
                            <p className="text-sm text-muted-foreground">
                                {t("Export your notes for backup or import from a JSON file.")}
                            </p>
                        </div>
                        <div className="flex gap-4">
                            <Button variant="outline" onClick={handleExport}>
                                {t("Export Data")}
                            </Button>
                            <Button variant="outline" onClick={() => fileInputRef.current?.click()}>
                                {t("Import Data")}
                            </Button>
                            <input
                                type="file"
                                ref={fileInputRef}
                                className="hidden"
                                accept=".json"
                                onChange={handleImport}
                            />
                        </div>
                    </div>
                </>}

                <DialogFooter>
                    <Button onClick={handleSave}>{t("Save changes")}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
