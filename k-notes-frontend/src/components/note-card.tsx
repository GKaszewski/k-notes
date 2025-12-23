import { type Note, useDeleteNote, useUpdateNote } from "@/hooks/use-notes";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Pin, Archive, Trash2, Edit } from "lucide-react";
import { format } from "date-fns";
import { toast } from "sonner";
import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { NoteForm } from "./note-form";
import ReactMarkdown from "react-markdown";
import { getNoteColor } from "@/lib/constants";
import clsx from "clsx";

interface NoteCardProps {
  note: Note;
}

export function NoteCard({ note }: NoteCardProps) {
  const { mutate: deleteNote } = useDeleteNote();
  const { mutate: updateNote } = useUpdateNote();
  const [editing, setEditing] = useState(false);

  // Archive toggle
  const toggleArchive = () => {
    updateNote({ 
        id: note.id,
        is_archived: !note.is_archived 
    });
  };
  
  // Pin toggle
  const togglePin = () => {
      updateNote({
          id: note.id,
          is_pinned: !note.is_pinned
      });
  };

  const handleDelete = () => {
      if (confirm("Are you sure?")) {
          deleteNote(note.id);
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
              toast.success("Note updated");
          }
      });
  }

  const colorClass = getNoteColor(note.color);

  return (
    <>
      <Card className={clsx(
          "relative group transition-all hover:shadow-md", 
          colorClass,
          note.is_pinned ? 'border-primary shadow-sm' : ''
      )}>
        <CardHeader className="pb-2">
          <div className="flex justify-between items-start">
            <CardTitle className="text-lg font-semibold line-clamp-1">{note.title}</CardTitle>
            {note.is_pinned && <Pin className="h-4 w-4 text-primary rotate-45" />}
          </div>
          <CardDescription className="text-xs opacity-70">
            {format(new Date(note.created_at), "MMM d, yyyy")}
          </CardDescription>
        </CardHeader>
        <CardContent className="pb-2">
          <div className="text-sm prose dark:prose-invert prose-sm max-w-none line-clamp-4">
            <ReactMarkdown>{note.content}</ReactMarkdown>
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
          <div className="flex justify-end w-full gap-1 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
            <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-black/5 dark:hover:bg-white/10" onClick={() => setEditing(true)}>
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
        <DialogContent>
            <DialogHeader>
                <DialogTitle>Edit Note</DialogTitle>
            </DialogHeader>
            <NoteForm 
                defaultValues={{
                    title: note.title,
                    content: note.content,
                    is_pinned: note.is_pinned,
                    color: note.color,
                    tags: note.tags.map(t => t.name).join(", "),
                }}
                onSubmit={handleEdit}
                submitLabel="Update"
             />
        </DialogContent>
      </Dialog>
    </>
  );
}
