import { useTranslation } from "react-i18next";
import { useApiUrl } from "@/hooks/use-api-url";
import { useDataManagement } from "@/hooks/use-data-management";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { LanguageSwitcher } from "@/components/language-switcher";
import { Settings as SettingsIcon, RotateCcw, Save, Download, Upload } from "lucide-react";

export default function SettingsPage() {
    const { t } = useTranslation();
    const { apiUrl, setApiUrl, currentApiUrl, saveApiUrl, resetApiUrl } = useApiUrl();
    const { fileInputRef, exportData, importData, triggerImport } = useDataManagement();

    return (
        <div className="max-w-2xl mx-auto space-y-6">
            <div className="flex items-center gap-3 mb-6">
                <SettingsIcon className="h-6 w-6" />
                <h1 className="text-3xl font-bold">{t("Settings")}</h1>
            </div>

            {/* API Configuration */}

            <Card>
                <CardHeader>
                    <CardTitle>{t("API Configuration")}</CardTitle>
                    <CardDescription>
                        {t("Configure the backend API URL for this application")}
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="space-y-2">
                        <Label className="text-sm text-muted-foreground">
                            {t("Current API URL")}
                        </Label>
                        <div className="p-3 bg-muted rounded-md font-mono text-sm break-all">
                            {currentApiUrl}
                        </div>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="api-url">{t("Custom API URL")}</Label>
                        <Input
                            id="api-url"
                            type="url"
                            placeholder="http://localhost:3000"
                            value={apiUrl}
                            onChange={(e) => setApiUrl(e.target.value)}
                            className="font-mono"
                        />
                        <p className="text-sm text-muted-foreground">
                            {t("Leave empty to use the default or Docker-injected URL")}
                        </p>
                    </div>

                    <div className="flex gap-2 pt-4">
                        <Button onClick={() => saveApiUrl(apiUrl)} className="flex items-center gap-2">
                            <Save className="h-4 w-4" />
                            {t("Save")}
                        </Button>
                        <Button
                            variant="outline"
                            onClick={resetApiUrl}
                            className="flex items-center gap-2"
                        >
                            <RotateCcw className="h-4 w-4" />
                            {t("Reset to Default")}
                        </Button>
                    </div>
                </CardContent>
            </Card>

            {/* Language Settings */}
            <Card>
                <CardHeader>
                    <CardTitle>{t("Language")}</CardTitle>
                    <CardDescription>
                        {t("Choose your preferred language")}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <LanguageSwitcher />
                </CardContent>
            </Card>

            {/* Data Management */}
            <Card>
                <CardHeader>
                    <CardTitle>{t("Data Management")}</CardTitle>
                    <CardDescription>
                        {t("Export your notes for backup or import from a JSON file.")}
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            onClick={exportData}
                            className="flex items-center gap-2"
                        >
                            <Download className="h-4 w-4" />
                            {t("Export Data")}
                        </Button>
                        <Button
                            variant="outline"
                            onClick={triggerImport}
                            className="flex items-center gap-2"
                        >
                            <Upload className="h-4 w-4" />
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
                </CardContent>
            </Card>
        </div>
    );
}
