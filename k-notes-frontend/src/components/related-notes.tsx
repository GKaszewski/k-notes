import { useRelatedNotes } from "@/hooks/use-related-notes";
import { useNotes } from "@/hooks/use-notes";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Link2 } from "lucide-react";

interface RelatedNotesProps {
    noteId: string;
    onSelectNote?: (id: string) => void;
}

export function RelatedNotes({ noteId, onSelectNote }: RelatedNotesProps) {
    const { relatedLinks, isRelatedLoading } = useRelatedNotes(noteId);
    const { data: notes } = useNotes(); // We need to look up note titles from source_id

    if (isRelatedLoading) {
        return (
            <div className="space-y-2 mt-4">
                <h3 className="text-sm font-medium">Related Notes</h3>
                <div className="flex flex-wrap gap-2">
                    <Skeleton className="h-8 w-24" />
                    <Skeleton className="h-8 w-32" />
                </div>
            </div>
        );
    }

    if (!relatedLinks || relatedLinks.length === 0) {
        return null;
    }

    return (
        <div className="space-y-2 mt-6 border-t pt-4">
            <h3 className="text-sm font-medium flex items-center gap-2">
                <Link2 className="w-4 h-4" />
                Related Notes
            </h3>
            <div className="flex flex-wrap gap-2">
                {relatedLinks.map((link) => {
                    const targetNote = notes?.find((n: any) => n.id === link.target_note_id);
                    if (!targetNote) return null;

                    return (
                        <Button
                            key={link.target_note_id}
                            variant="outline"
                            size="sm"
                            className="h-8 text-xs max-w-[200px] justify-start"
                            onClick={() => onSelectNote?.(link.target_note_id)}
                        >
                            <span className="truncate">{targetNote.title || "Untitled"}</span>
                            <Badge variant="secondary" className="ml-2 text-[10px] h-5 px-1">
                                {Math.round(link.score * 100)}%
                            </Badge>
                        </Button>
                    );
                })}
            </div>
        </div>
    );
}
