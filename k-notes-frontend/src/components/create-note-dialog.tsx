import { useState } from "react";
import { useCreateNote } from "@/hooks/use-notes";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { NoteForm } from "./note-form";
import { toast } from "sonner";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

interface CreateNoteDialogProps {
  trigger?: React.ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function CreateNoteDialog({ trigger, open: controlledOpen, onOpenChange }: CreateNoteDialogProps) {
  const [internalOpen, setInternalOpen] = useState(false);
  const { mutate: createNote, isPending } = useCreateNote();
  const { t } = useTranslation();

  // Support both controlled and uncontrolled modes
  const isControlled = controlledOpen !== undefined;
  const open = isControlled ? controlledOpen : internalOpen;
  const setOpen = isControlled ? (onOpenChange ?? (() => { })) : setInternalOpen;

  const onSubmit = (data: any) => {
    // Parse tags
    const tags = data.tags
      ? data.tags.split(",").map((t: string) => t.trim()).filter(Boolean)
      : [];

    createNote({ ...data, tags }, {
      onSuccess: () => {
        toast.success(t("Note created"));
        setOpen(false);
      },
      onError: (error: any) => {
        toast.error(error.message);
      }
    });
  };

  const defaultTrigger = (
    <Button>
      <Plus className="mr-2 h-4 w-4" />
      {t("New Note")}
    </Button>
  );

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {trigger !== undefined && (
        <DialogTrigger asChild>
          {trigger ?? defaultTrigger}
        </DialogTrigger>
      )}
      {trigger === undefined && (
        <DialogTrigger asChild>
          {defaultTrigger}
        </DialogTrigger>
      )}
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{t("Create Note")}</DialogTitle>
          <DialogDescription>
            {t("Add a new note to your collection.")}
          </DialogDescription>
        </DialogHeader>
        <NoteForm onSubmit={onSubmit} isLoading={isPending} submitLabel={t("Create")} />
      </DialogContent>
    </Dialog>
  );
}
