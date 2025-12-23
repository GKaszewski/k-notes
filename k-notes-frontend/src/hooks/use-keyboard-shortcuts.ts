import { useEffect, useCallback } from "react";

interface KeyboardShortcutsConfig {
    onNewNote?: () => void;
    onFocusSearch?: () => void;
    onEscape?: () => void;
}

export function useKeyboardShortcuts(config: KeyboardShortcutsConfig) {
    const { onNewNote, onFocusSearch, onEscape } = config;

    const handleKeyDown = useCallback((event: KeyboardEvent) => {
        // Don't trigger shortcuts when typing in inputs/textareas
        const target = event.target as HTMLElement;
        const isInputField =
            target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable;

        // Escape should always work (to close dialogs)
        if (event.key === "Escape") {
            onEscape?.();
            return;
        }

        // Other shortcuts only work when not in an input field
        if (isInputField) return;

        // 'n' for new note
        if (event.key === "n" && !event.metaKey && !event.ctrlKey) {
            event.preventDefault();
            onNewNote?.();
        }

        // '/' to focus search
        if (event.key === "/") {
            event.preventDefault();
            onFocusSearch?.();
        }
    }, [onNewNote, onFocusSearch, onEscape]);

    useEffect(() => {
        document.addEventListener("keydown", handleKeyDown);
        return () => document.removeEventListener("keydown", handleKeyDown);
    }, [handleKeyDown]);
}
