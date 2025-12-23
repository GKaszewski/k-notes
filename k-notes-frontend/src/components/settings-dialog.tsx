import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";

interface SettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
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
                <DialogFooter>
                    <Button onClick={handleSave}>Save changes</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
