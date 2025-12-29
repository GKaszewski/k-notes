
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import { SlashCommand } from './extensions';
import { getSuggestionItems, renderItems } from './suggestions';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import { cn } from '@/lib/utils';
import { useEffect } from 'react';

interface EditorProps {
    value?: string;
    onChange?: (value: string) => void;
    placeholder?: string;
    className?: string;
}

export function Editor({ value, onChange, placeholder, className }: EditorProps) {
    const editor = useEditor({
        extensions: [
            StarterKit.configure({
                bulletList: {
                    keepMarks: true,
                    keepAttributes: false,
                },
                orderedList: {
                    keepMarks: true,
                    keepAttributes: false,
                },
            }),
            Placeholder.configure({
                placeholder: placeholder || 'Type / for commands...',
            }),
            Markdown,
            TaskList,
            TaskItem.configure({
                nested: true,
            }),
            SlashCommand.configure({
                suggestion: {
                    items: getSuggestionItems,
                    render: renderItems,
                },
            }),
        ],
        content: value,
        editorProps: {
            attributes: {
                class: cn(
                    "min-h-[100px] max-h-[400px] overflow-y-auto w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 prose dark:prose-invert max-w-none break-all min-w-0",
                    className
                ),
            },
        },
        onUpdate: ({ editor }) => {
            const markdown = (editor.storage as any).markdown.getMarkdown();
            onChange?.(markdown);
        },
    });

    // Sync content if value changes externally (and editor is not focused? care to avoid loops)
    useEffect(() => {
        if (editor && value !== undefined && value !== (editor.storage as any).markdown.getMarkdown()) {
            // Only set content if it's different to avoid cursor jumping
            // A simple check might not be enough but good for now for initial loading
            if (editor.getText() === "" && value !== "") {
                editor.commands.setContent(value);
            }
        }
    }, [value, editor]);


    return (
        <div className="relative w-full">
            <EditorContent editor={editor} />
            {/* 
        TipTap styles for placeholder
      */}
            <style>{`
        .tiptap p.is-editor-empty:first-child::before {
          color: #adb5bd;
          content: attr(data-placeholder);
          float: left;
          height: 0;
          pointer-events: none;
        }
        
        /* Basic Task list styles */
        ul[data-type="taskList"] {
            list-style: none;
            padding: 0;
        }
        
        ul[data-type="taskList"] li {
            display: flex;
            align-items: flex-start; /* Align checkbox with top of text */
        }
        
        ul[data-type="taskList"] li > label {
            flex: 0 0 auto;
            margin-right: 0.5rem;
            user-select: none;
            margin-top: 0.15rem; /* Optical alignment */
        }
        
        ul[data-type="taskList"] li > div {
            flex: 1 1 auto;
        }
      `}</style>
        </div>
    );
}
