import { useState } from "react";
import { useCreateNote } from "@/hooks/use-notes";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { NoteForm } from "./note-form";
import { toast } from "sonner";
import { Plus } from "lucide-react";

export function CreateNoteDialog() {
  const [open, setOpen] = useState(false);
  const { mutate: createNote, isPending } = useCreateNote();

  const onSubmit = (data: any) => {
    // Parse tags
    const tags = data.tags
        ? data.tags.split(",").map((t: string) => t.trim()).filter(Boolean)
        : [];
        
    createNote({ ...data, tags }, {
      onSuccess: () => {
        toast.success("Note created");
        setOpen(false);
      },
      onError: (error: any) => {
        toast.error(error.message);
      }
    });
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
            <Plus className="mr-2 h-4 w-4" />
            New Note
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create Note</DialogTitle>
          <DialogDescription>
            Add a new note to your collection.
          </DialogDescription>
        </DialogHeader>
        <NoteForm onSubmit={onSubmit} isLoading={isPending} submitLabel="Create" />
      </DialogContent>
    </Dialog>
  );
}
