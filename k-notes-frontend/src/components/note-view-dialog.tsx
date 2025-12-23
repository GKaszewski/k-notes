import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { format } from "date-fns";
import ReactMarkdown from "react-markdown";
import { type Note } from "@/hooks/use-notes";
import { Edit, Calendar, Pin } from "lucide-react";
import { getNoteColor } from "@/lib/constants";
import clsx from "clsx";


interface NoteViewDialogProps {
    note: Note;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onEdit: () => void;
}

export function NoteViewDialog({ note, open, onOpenChange, onEdit }: NoteViewDialogProps) {
    const colorClass = getNoteColor(note.color);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className={clsx("max-w-3xl max-h-[85vh] flex flex-col p-6 gap-0 overflow-hidden", colorClass)}>
                <DialogHeader className="pb-4 shrink-0">
                    <div className="flex justify-between items-start gap-4">
                        <DialogTitle className="text-2xl font-bold leading-tight break-words">
                            {note.title}
                        </DialogTitle>
                        {note.is_pinned && (
                            <Pin className="h-5 w-5 text-primary rotate-45 shrink-0" />
                        )}
                    </div>
                    <div className="flex items-center text-sm text-muted-foreground gap-2 mt-1">
                        <Calendar className="h-3.5 w-3.5" />
                        <span>Created {format(new Date(note.created_at), "MMMM d, yyyy 'at' h:mm a")}</span>
                    </div>
                </DialogHeader>

                <div className="flex-1 min-h-0 overflow-y-auto -mx-6 px-6">
                    <div className="prose dark:prose-invert max-w-none text-base leading-relaxed break-words pb-6">
                        <ReactMarkdown>{note.content}</ReactMarkdown>
                    </div>
                </div>

                <DialogFooter className="pt-4 mt-2 border-t border-black/5 dark:border-white/5 flex sm:justify-between items-center gap-4 shrink-0">
                    <div className="flex flex-wrap gap-1.5 flex-1">
                        {note.tags.map(tag => (
                            <Badge key={tag.id} variant="secondary" className="bg-black/5 hover:bg-black/10 dark:bg-white/10 dark:hover:bg-white/20">
                                {tag.name}
                            </Badge>
                        ))}
                    </div>
                    <Button onClick={() => {
                        onOpenChange(false);
                        onEdit();
                    }}>
                        <Edit className="h-4 w-4 mr-2" />
                        Edit Note
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
