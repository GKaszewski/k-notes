import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "@/components/language-switcher";
import { useApiUrl } from "@/hooks/use-api-url";
import { useDataManagement } from "@/hooks/use-data-management";

interface SettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    dataManagementEnabled: boolean;
}

export function SettingsDialog({ open, onOpenChange, dataManagementEnabled }: SettingsDialogProps) {
    const { t } = useTranslation();
    const { apiUrl, setApiUrl, saveApiUrl } = useApiUrl();
    const { fileInputRef, exportData, importData, triggerImport } = useDataManagement();

    const handleSave = () => {
        if (saveApiUrl(apiUrl)) {
            onOpenChange(false);
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
                            value={apiUrl}
                            onChange={(e) => setApiUrl(e.target.value)}
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
                            <Button variant="outline" onClick={exportData}>
                                {t("Export Data")}
                            </Button>
                            <Button variant="outline" onClick={triggerImport}>
                                {t("Import Data")}
                            </Button>
                            <input
                                type="file"
                                ref={fileInputRef}
                                className="hidden"
                                accept=".json"
                                onChange={importData}
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
