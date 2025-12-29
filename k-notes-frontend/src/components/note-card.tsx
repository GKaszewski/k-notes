import { type Note, useDeleteNote, useUpdateNote } from "@/hooks/use-notes";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Pin, Archive, Trash2, Edit, History, Copy } from "lucide-react";
import { format } from "date-fns";
import { toast } from "sonner";
import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { NoteForm } from "./note-form";
import ReactMarkdown from "react-markdown";
import { getNoteColor } from "@/lib/constants";
import clsx from "clsx";
import remarkGfm from "remark-gfm";
import { VersionHistoryDialog } from "./version-history-dialog";
import { NoteViewDialog } from "./note-view-dialog";
import { Checkbox } from "@/components/ui/checkbox";
import { useBulkSelection } from "@/components/bulk-selection-context";
import { useTranslation } from "react-i18next";

interface NoteCardProps {
  note: Note;
}

export function NoteCard({ note }: NoteCardProps) {
  const { mutate: deleteNote } = useDeleteNote();
  const { mutate: updateNote } = useUpdateNote();
  const [editing, setEditing] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [viewOpen, setViewOpen] = useState(false);
  const { t } = useTranslation();

  // Bulk selection
  const { isSelected, toggleSelection, isBulkMode } = useBulkSelection();
  const selected = isSelected(note.id);

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleSelection(note.id);
  };

  // Archive toggle
  const toggleArchive = (e: React.MouseEvent) => {
    e.stopPropagation();
    updateNote({
      id: note.id,
      is_archived: !note.is_archived
    });
  };

  // Pin toggle
  const togglePin = (e: React.MouseEvent) => {
    e.stopPropagation();
    updateNote({
      id: note.id,
      is_pinned: !note.is_pinned
    });
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(t("Are you sure?"))) {
      deleteNote(note.id);
    }
  }

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const textToCopy = `${note.title}\n\n${note.content}`;
      await navigator.clipboard.writeText(textToCopy);
      toast.success(t("Note copied to clipboard"));
    } catch (err) {
      toast.error(t("Failed to copy note"));
    }
  }

  const handleEdit = (data: any) => {
    const tags = data.tags
      ? data.tags.split(",").map((t: string) => t.trim()).filter(Boolean)
      : [];

    updateNote({
      id: note.id,
      ...data,
      tags,
    }, {
      onSuccess: () => {
        setEditing(false);
        toast.success(t("Note updated"));
      }
    });
  }

  const colorClass = getNoteColor(note.color);

  return (
    <>
      <Card
        className={clsx(
          "relative group transition-all hover:shadow-md cursor-pointer",
          colorClass,
          note.is_pinned ? 'border-primary shadow-sm' : '',
          selected && 'ring-2 ring-primary ring-offset-2'
        )}
        onClick={() => !isBulkMode && setViewOpen(true)}
      >
        {/* Bulk selection checkbox */}
        <div
          className={clsx(
            "absolute top-2 left-2 z-10 transition-opacity",
            isBulkMode ? "opacity-100" : "opacity-0 group-hover:opacity-100"
          )}
          onClick={handleCheckboxClick}
        >
          <Checkbox
            checked={selected}
            className="h-5 w-5 bg-background/80 backdrop-blur-sm border-2"
          />
        </div>

        <CardHeader className="pb-2">
          <div className="flex justify-between items-start">
            <CardTitle className={clsx("text-lg font-semibold line-clamp-1", isBulkMode && "pl-6")}>{note.title}</CardTitle>
            {note.is_pinned && <Pin className="h-4 w-4 text-primary rotate-45" />}
          </div>
          <CardDescription className="text-xs opacity-70">
            {format(new Date(note.created_at), "MMM d, yyyy")}
          </CardDescription>
        </CardHeader>
        <CardContent className="pb-2">
          <div className="text-sm prose dark:prose-invert prose-sm max-w-none line-clamp-4">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{note.content}</ReactMarkdown>
          </div>
        </CardContent>
        <CardFooter className="flex flex-col items-start gap-2 pt-2">
          <div className="flex flex-wrap gap-1">
            {note.tags.map(tag => (
              <Badge key={tag.id} variant="secondary" className="text-xs bg-black/5 hover:bg-black/10 dark:bg-white/10 dark:hover:bg-white/20">
                {tag.name}
              </Badge>
            ))}
          </div>
          <div className="flex justify-end w-full gap-1 opacity-100 lg:opacity-0 lg:group-hover:opacity-100 transition-opacity">
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={(e) => { e.stopPropagation(); setHistoryOpen(true); }} title={t("History")}>
              <History className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={handleCopy} title={t("Copy note")}>
              <Copy className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={(e) => { e.stopPropagation(); setEditing(true); }}>
              <Edit className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={togglePin}>
              <Pin className={`h-4 w-4 ${note.is_pinned ? 'fill-current' : ''}`} />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={toggleArchive}>
              <Archive className={`h-4 w-4 ${note.is_archived ? 'fill-current' : ''}`} />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={handleDelete}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </CardFooter>
      </Card>

      <Dialog open={editing} onOpenChange={setEditing}>
        <DialogContent className="max-w-3xl max-h-[85vh] flex flex-col p-6 gap-0 overflow-hidden">
          <DialogHeader className="pb-4 shrink-0">
            <DialogTitle>{t("Edit Note")}</DialogTitle>
          </DialogHeader>
          <div className="flex-1 min-h-0 overflow-y-auto -mx-6 px-6">
            <NoteForm
              defaultValues={{
                title: note.title,
                content: note.content,
                is_pinned: note.is_pinned,
                color: note.color,
                tags: note.tags.map(t => t.name).join(", "),
              }}
              onSubmit={handleEdit}
              submitLabel={t("Update")}
            />
          </div>
        </DialogContent>
      </Dialog>

      <VersionHistoryDialog
        open={historyOpen}
        onOpenChange={setHistoryOpen}
        noteId={note.id}
        noteTitle={note.title}
      />

      <NoteViewDialog
        open={viewOpen}
        onOpenChange={setViewOpen}
        note={note}
        onEdit={() => setEditing(true)}
      />
    </>
  );
}

