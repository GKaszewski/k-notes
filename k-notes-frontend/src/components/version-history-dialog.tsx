import { ScrollArea } from "@/components/ui/scroll-area";
import { useNoteVersions, type NoteVersion, useUpdateNote } from "@/hooks/use-notes";
import { formatDistanceToNow, format } from "date-fns";
import { Loader2, History, Download, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";

interface VersionHistoryDialogProps {
    noteId: string;
    noteTitle: string;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function VersionHistoryDialog({
    noteId,
    noteTitle,
    open,
    onOpenChange,
}: VersionHistoryDialogProps) {
    const { data: versions, isLoading } = useNoteVersions(noteId, open);
    const { mutate: updateNote } = useUpdateNote();

    const handleDownload = (version: NoteVersion) => {
        const text = `${version.title}\n\n${version.content}`;
        const blob = new Blob([text], { type: "text/plain" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${version.title.replace(/[^a-z0-9]/gi, '_').toLowerCase()}-${format(new Date(version.created_at), "yyyy-MM-dd-HH-mm")}.txt`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        toast.success("Downloaded version");
    };

    const handleRestore = (version: NoteVersion) => {
        if (confirm("Are you sure you want to restore this version? The current version will be saved as a new history entry.")) {
            updateNote({
                id: noteId,
                title: version.title,
                content: version.content,
            }, {
                onSuccess: () => {
                    toast.success("Version restored");
                    onOpenChange(false);
                }
            });
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl h-[80vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <History className="h-5 w-5" />
                        Version History
                    </DialogTitle>
                    <DialogDescription>
                        History for "{noteTitle}"
                    </DialogDescription>
                </DialogHeader>

                <div className="flex-1 min-h-0 overflow-hidden">
                    <ScrollArea className="h-full">
                        <div className="pr-4">
                            {isLoading ? (
                                <div className="flex justify-center p-4">
                                    <Loader2 className="h-6 w-6 animate-spin" />
                                </div>
                            ) : versions?.length === 0 ? (
                                <div className="text-center text-muted-foreground p-4">
                                    No history available for this note.
                                </div>
                            ) : (
                                <div className="space-y-4 p-1">
                                    {(versions as NoteVersion[])?.map((version) => (
                                        <div
                                            key={version.id}
                                            className="border rounded-lg p-4 space-y-3 bg-card"
                                        >
                                            <div className="flex justify-between items-center text-sm text-muted-foreground">
                                                <div className="flex items-center gap-2">
                                                    <span className="font-medium text-foreground">
                                                        {format(new Date(version.created_at), "MMM d, yyyy HH:mm")}
                                                    </span>
                                                    <span>
                                                        ({formatDistanceToNow(new Date(version.created_at), {
                                                            addSuffix: true,
                                                        })})
                                                    </span>
                                                </div>
                                                <div className="flex gap-1">
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        className="h-8 gap-1.5"
                                                        onClick={() => handleDownload(version)}
                                                    >
                                                        <Download className="h-3.5 w-3.5" />
                                                        Download
                                                    </Button>
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        className="h-8 gap-1.5"
                                                        onClick={() => handleRestore(version)}
                                                    >
                                                        <RotateCcw className="h-3.5 w-3.5" />
                                                        Restore
                                                    </Button>
                                                </div>
                                            </div>
                                            <div className="font-medium leading-none">{version.title}</div>
                                            <div className="text-sm whitespace-pre-wrap font-mono bg-muted/50 p-3 rounded-md border">
                                                {version.content}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    </ScrollArea>
                </div>
            </DialogContent>
        </Dialog>
    );
}
