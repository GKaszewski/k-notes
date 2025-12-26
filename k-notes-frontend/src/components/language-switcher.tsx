import { useTranslation } from "react-i18next";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Languages } from "lucide-react";

const LANGUAGES = [
    { code: "en", label: "English" },
    { code: "pl", label: "Polski" },
    { code: "es", label: "Español" },
    { code: "de", label: "Deutsch" },
    { code: "fr", label: "Français" },
];

export function LanguageSwitcher() {
    const { i18n, t } = useTranslation();

    const changeLanguage = (languageCode: string) => {
        i18n.changeLanguage(languageCode);
    };

    return (
        <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="language" className="text-right flex items-center gap-2">
                    <Languages className="h-4 w-4" />
                    {t("Language")}
                </Label>
                <div className="col-span-3 flex gap-2 flex-wrap">
                    {LANGUAGES.map((lang) => (
                        <Button
                            key={lang.code}
                            variant={i18n.language === lang.code ? "default" : "outline"}
                            size="sm"
                            onClick={() => changeLanguage(lang.code)}
                        >
                            {lang.label}
                        </Button>
                    ))}
                </div>
            </div>
        </div>
    );
}
