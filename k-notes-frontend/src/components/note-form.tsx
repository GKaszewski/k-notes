import { useForm } from "react-hook-form";
import { NOTE_COLORS } from "@/lib/constants";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Checkbox } from "@/components/ui/checkbox";
import { Editor } from "@/components/editor/editor";
import { useTranslation } from "react-i18next";

const noteSchema = (t: any) => z.object({
  title: z.string().min(1, t("Title is required")).max(200, t("Title too long")),
  content: z.string().optional(),
  is_pinned: z.boolean().default(false),
  tags: z.string().optional(), // Comma separated for now
  color: z.string().default("DEFAULT"),
});

type NoteFormValues = z.infer<ReturnType<typeof noteSchema>>;

interface NoteFormProps {
  defaultValues?: Partial<NoteFormValues>;
  onSubmit: (data: NoteFormValues) => void;
  isLoading?: boolean;
  submitLabel?: string;
}

export function NoteForm({ defaultValues, onSubmit, isLoading, submitLabel = "Save" }: NoteFormProps) {
  const { t } = useTranslation();
  const form = useForm<NoteFormValues>({
    resolver: zodResolver(noteSchema(t)) as any,
    defaultValues: {
      title: "",
      content: "",
      is_pinned: false,
      tags: "",
      color: "DEFAULT",
      ...defaultValues,
    },
  });

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit as any)} className="space-y-4">
        <FormField
          control={form.control as any}
          name="title"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("Title")}</FormLabel>
              <FormControl>
                <Input placeholder={t("Note title")} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control as any}
          name="content"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("Content")}</FormLabel>
              <FormControl>
                <Editor
                  placeholder={t("Note content... Type / for commands")}
                  value={field.value}
                  onChange={field.onChange}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control as any}
          name="tags"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("Tags (comma separated)")}</FormLabel>
              <FormControl>
                <Input placeholder={t("work, todo, ideas")} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control as any}
          name="color"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("Color")}</FormLabel>
              <FormControl>
                <div className="flex gap-2 flex-wrap">
                  {NOTE_COLORS.map((color) => (
                    <div
                      key={color.name}
                      onClick={() => field.onChange(color.name)}
                      className={`w-8 h-8 rounded-full cursor-pointer border-2 transition-all ${color.value.split(" ")[0] // Take background class
                        } ${field.value === color.name
                          ? "border-primary scale-110"
                          : "border-transparent hover:scale-105"
                        }`}
                      title={color.label}
                    />
                  ))}
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control as any}
          name="is_pinned"
          render={({ field }) => (
            <FormItem className="flex flex-row items-center space-x-3 space-y-0 rounded-md border p-4">
              <FormControl>
                <Checkbox
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              </FormControl>
              <div className="space-y-1 leading-none">
                <FormLabel>{t("Pin this note")}</FormLabel>
              </div>
            </FormItem>
          )}
        />
        <Button type="submit" disabled={isLoading} className="w-full">
          {isLoading ? t("Saving...") : submitLabel}
        </Button>
      </form>
    </Form>
  );
}
