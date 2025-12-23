import { useRef, useState, useEffect } from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { Separator } from "@/components/ui/separator";

interface SettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    dataManagementEnabled: boolean;
}

export function SettingsDialog({ open, onOpenChange, dataManagementEnabled }: SettingsDialogProps) {
    const [url, setUrl] = useState("http://localhost:3000");

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
            toast.success("Settings saved. Please refresh the page.");
            onOpenChange(false);
            window.location.reload();
        } catch (e) {
            toast.error("Invalid URL");
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
            toast.success("Export successful");
        } catch (e) {
            toast.error("Export failed");
        }
    };

    const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        try {
            const text = await file.text();
            const data = JSON.parse(text);
            await api.importData(data);
            toast.success("Import successful. Reloading...");
            onOpenChange(false);
            window.location.reload();
        } catch (e) {
            console.error(e);
            toast.error("Import failed");
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Settings</DialogTitle>
                    <DialogDescription>
                        Configure the application settings.
                    </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Label htmlFor="url" className="text-right">
                            Backend URL
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

                {dataManagementEnabled && <>
                    <Separator className="my-2" />

                    <div className="py-4 space-y-4">
                        <div className="flex flex-col space-y-2">
                            <h4 className="font-medium leading-none">Data Management</h4>
                            <p className="text-sm text-muted-foreground">
                                Export your notes for backup or import from a JSON file.
                            </p>
                        </div>
                        <div className="flex gap-4">
                            <Button variant="outline" onClick={handleExport}>
                                Export Data
                            </Button>
                            <Button variant="outline" onClick={() => fileInputRef.current?.click()}>
                                Import Data
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
                    <Button onClick={handleSave}>Save changes</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
