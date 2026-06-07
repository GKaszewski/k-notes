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
    <Button className="rounded-full px-5 aero-aqua-btn">
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
      <DialogContent className="sm:max-w-[425px] max-h-[90dvh] flex flex-col gap-0 p-0">
        <div className="px-6 pt-6 pb-4 shrink-0">
          <DialogHeader>
            <DialogTitle>{t("Create Note")}</DialogTitle>
            <DialogDescription>
              {t("Add a new note to your collection.")}
            </DialogDescription>
          </DialogHeader>
        </div>
        {/* Scrollable form body — ensures the submit button stays reachable when the
            iOS keyboard pushes the viewport up. dvh accounts for the keyboard height. */}
        <div className="flex-1 min-h-0 overflow-y-auto px-6 pb-6">
          <NoteForm onSubmit={onSubmit} isLoading={isPending} submitLabel={t("Create")} />
        </div>
      </DialogContent>
    </Dialog>
  );
}
