import { createContext, useContext, useState, useCallback, type ReactNode } from "react";

interface BulkSelectionContextType {
    selectedIds: Set<string>;
    isBulkMode: boolean;
    toggleSelection: (id: string) => void;
    selectAll: (ids: string[]) => void;
    clearSelection: () => void;
    isSelected: (id: string) => boolean;
}

const BulkSelectionContext = createContext<BulkSelectionContextType | null>(null);

export function BulkSelectionProvider({ children }: { children: ReactNode }) {
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

    const toggleSelection = useCallback((id: string) => {
        setSelectedIds((prev) => {
            const next = new Set(prev);
            if (next.has(id)) {
                next.delete(id);
            } else {
                next.add(id);
            }
            return next;
        });
    }, []);

    const selectAll = useCallback((ids: string[]) => {
        setSelectedIds(new Set(ids));
    }, []);

    const clearSelection = useCallback(() => {
        setSelectedIds(new Set());
    }, []);

    const isSelected = useCallback((id: string) => {
        return selectedIds.has(id);
    }, [selectedIds]);

    const isBulkMode = selectedIds.size > 0;

    return (
        <BulkSelectionContext.Provider
            value={{
                selectedIds,
                isBulkMode,
                toggleSelection,
                selectAll,
                clearSelection,
                isSelected,
            }}
        >
            {children}
        </BulkSelectionContext.Provider>
    );
}

export function useBulkSelection() {
    const context = useContext(BulkSelectionContext);
    if (!context) {
        throw new Error("useBulkSelection must be used within a BulkSelectionProvider");
    }
    return context;
}
