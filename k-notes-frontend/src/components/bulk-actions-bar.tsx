import { useBulkSelection } from "@/components/bulk-selection-context";
import { useDeleteNote, useUpdateNote } from "@/hooks/use-notes";
import { Button } from "@/components/ui/button";
import { Archive, Trash2, X } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

export function BulkActionsBar() {
    const { selectedIds, clearSelection, isBulkMode } = useBulkSelection();
    const { mutate: deleteNote } = useDeleteNote();
    const { mutate: updateNote } = useUpdateNote();
    const { t } = useTranslation();

    if (!isBulkMode) return null;

    const handleArchiveAll = () => {
        const ids = Array.from(selectedIds);
        ids.forEach((id) => {
            updateNote({ id, is_archived: true });
        });
        toast.success(t("Archived {{count}} note", { count: ids.length, defaultValue_other: "Archived {{count}} notes" }));
        clearSelection();
    };

    const handleDeleteAll = () => {
        if (!confirm(t("Are you sure you want to delete {{count}} note?", { count: selectedIds.size, defaultValue_other: "Are you sure you want to delete {{count}} notes?" }))) return;

        const ids = Array.from(selectedIds);
        ids.forEach((id) => {
            deleteNote(id);
        });
        toast.success(t("Deleted {{count}} note", { count: ids.length, defaultValue_other: "Deleted {{count}} notes" }));
        clearSelection();
    };

    return (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 bg-background border rounded-full px-4 py-2 shadow-lg animate-in slide-in-from-bottom-4 duration-200">
            <span className="text-sm font-medium">
                {t("{{count}} selected", { count: selectedIds.size })}
            </span>

            <div className="h-4 w-px bg-border" />

            <Button
                variant="ghost"
                size="sm"
                onClick={handleArchiveAll}
                className="gap-2"
            >
                <Archive className="h-4 w-4" />
                {t("Archive")}
            </Button>

            <Button
                variant="ghost"
                size="sm"
                onClick={handleDeleteAll}
                className="gap-2 text-destructive hover:text-destructive hover:bg-destructive/10"
            >
                <Trash2 className="h-4 w-4" />
                {t("Delete")}
            </Button>

            <div className="h-4 w-px bg-border" />

            <Button
                variant="ghost"
                size="icon"
                onClick={clearSelection}
                className="h-8 w-8"
            >
                <X className="h-4 w-4" />
            </Button>
        </div>
    );
}
