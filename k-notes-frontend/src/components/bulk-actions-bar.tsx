import { useBulkSelection } from "@/components/bulk-selection-context";
import { useDeleteNote, useUpdateNote } from "@/hooks/use-notes";
import { Button } from "@/components/ui/button";
import { Archive, Trash2, X } from "lucide-react";
import { toast } from "sonner";

export function BulkActionsBar() {
    const { selectedIds, clearSelection, isBulkMode } = useBulkSelection();
    const { mutate: deleteNote } = useDeleteNote();
    const { mutate: updateNote } = useUpdateNote();

    if (!isBulkMode) return null;

    const handleArchiveAll = () => {
        const ids = Array.from(selectedIds);
        ids.forEach((id) => {
            updateNote({ id, is_archived: true });
        });
        toast.success(`Archived ${ids.length} note${ids.length > 1 ? "s" : ""}`);
        clearSelection();
    };

    const handleDeleteAll = () => {
        if (!confirm(`Are you sure you want to delete ${selectedIds.size} note(s)?`)) return;

        const ids = Array.from(selectedIds);
        ids.forEach((id) => {
            deleteNote(id);
        });
        toast.success(`Deleted ${ids.length} note${ids.length > 1 ? "s" : ""}`);
        clearSelection();
    };

    return (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 bg-background border rounded-full px-4 py-2 shadow-lg animate-in slide-in-from-bottom-4 duration-200">
            <span className="text-sm font-medium">
                {selectedIds.size} selected
            </span>

            <div className="h-4 w-px bg-border" />

            <Button
                variant="ghost"
                size="sm"
                onClick={handleArchiveAll}
                className="gap-2"
            >
                <Archive className="h-4 w-4" />
                Archive
            </Button>

            <Button
                variant="ghost"
                size="sm"
                onClick={handleDeleteAll}
                className="gap-2 text-destructive hover:text-destructive hover:bg-destructive/10"
            >
                <Trash2 className="h-4 w-4" />
                Delete
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
