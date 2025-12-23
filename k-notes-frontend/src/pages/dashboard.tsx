import { useState, useRef } from "react";
import { useNotes, useSearchNotes, type Note } from "@/hooks/use-notes";
import { CreateNoteDialog } from "@/components/create-note-dialog";
import { NoteCard } from "@/components/note-card";
import { Input } from "@/components/ui/input";
import { Search, LayoutGrid, List, Plus, Pin, X } from "lucide-react";
import { useLocation, useSearchParams, Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import clsx from "clsx";
import Masonry from "react-masonry-css";
import { NoteCardSkeletonGrid } from "@/components/note-card-skeleton";
import { Badge } from "@/components/ui/badge";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";

// Masonry breakpoint columns configuration
const masonryBreakpoints = {
    default: 4,
    1280: 4,
    1024: 3,
    768: 2,
    640: 1,
};

export default function DashboardPage() {
    const location = useLocation();
    const [searchParams] = useSearchParams();
    const isArchive = location.pathname === "/archive";
    const activeTag = searchParams.get("tag");

    // View mode state
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");

    // Search state
    const [searchQuery, setSearchQuery] = useState("");
    const searchInputRef = useRef<HTMLInputElement>(null);

    // Create note dialog state (keyboard controlled)
    const [createNoteOpen, setCreateNoteOpen] = useState(false);

    // Keyboard shortcuts
    useKeyboardShortcuts({
        onNewNote: () => !isArchive && setCreateNoteOpen(true),
        onFocusSearch: () => searchInputRef.current?.focus(),
        onEscape: () => {
            searchInputRef.current?.blur();
            setCreateNoteOpen(false);
        },
    });

    // Fetch notes with optional tag filter
    const { data: notes, isLoading: notesLoading } = useNotes(
        searchQuery ? undefined : { archived: isArchive, tag: activeTag ?? undefined }
    );

    // Fetch search results if searching
    const { data: searchResults, isLoading: searchLoading } = useSearchNotes(searchQuery);

    const displayNotes = searchQuery ? searchResults : notes;
    const isLoading = searchQuery ? searchLoading : notesLoading;

    // Separate pinned and unpinned notes
    const pinnedNotes = !searchQuery && !isArchive
        ? (displayNotes?.filter((n: Note) => n.is_pinned) ?? [])
        : [];
    const unpinnedNotes = displayNotes?.filter((n: Note) => searchQuery || isArchive || !n.is_pinned) ?? [];

    const renderNotes = (notesList: Note[]) => {
        if (viewMode === "list") {
            return (
                <div className="flex flex-col gap-4 max-w-3xl mx-auto">
                    {notesList.map((note: Note) => (
                        <NoteCard key={note.id} note={note} />
                    ))}
                </div>
            );
        }

        return (
            <Masonry
                breakpointCols={masonryBreakpoints}
                className="flex -ml-4 w-auto"
                columnClassName="pl-4 bg-clip-padding"
            >
                {notesList.map((note: Note) => (
                    <div key={note.id} className="mb-4">
                        <NoteCard note={note} />
                    </div>
                ))}
            </Masonry>
        );
    };

    return (
        <div className="max-w-7xl mx-auto pb-20 md:pb-0">
            {/* Action Bar */}
            <div className="flex flex-col md:flex-row gap-4 justify-between items-center mb-6">
                <div className="relative w-full md:w-96">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input
                        ref={searchInputRef}
                        id="search-input"
                        placeholder="Search your notes..."
                        className="pl-9 w-full bg-background"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                    />
                </div>

                <div className="flex items-center gap-2">
                    <div className="flex items-center bg-muted/50 p-1 rounded-lg border">
                        <Button
                            variant="ghost"
                            size="icon"
                            className={clsx("h-8 w-8", viewMode === "grid" && "bg-background shadow-sm")}
                            onClick={() => setViewMode("grid")}
                            title="Grid View"
                        >
                            <LayoutGrid className="h-4 w-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            className={clsx("h-8 w-8", viewMode === "list" && "bg-background shadow-sm")}
                            onClick={() => setViewMode("list")}
                            title="List View"
                        >
                            <List className="h-4 w-4" />
                        </Button>
                    </div>
                    {!isArchive && (
                        <div className="hidden md:block">
                            <CreateNoteDialog open={createNoteOpen} onOpenChange={setCreateNoteOpen} />
                        </div>
                    )}
                </div>
            </div>

            {/* Active Tag Filter Badge */}
            {activeTag && (
                <div className="flex items-center gap-2 mb-4">
                    <span className="text-sm text-muted-foreground">Filtering by:</span>
                    <Badge variant="secondary" className="flex items-center gap-1">
                        {activeTag}
                        <Link to="/" className="ml-1 hover:text-destructive">
                            <X className="h-3 w-3" />
                        </Link>
                    </Badge>
                </div>
            )}

            {/* Loading State - Skeleton */}
            {isLoading && (
                <div className={clsx(
                    viewMode === "grid"
                        ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
                        : "flex flex-col gap-4 max-w-3xl mx-auto"
                )}>
                    <NoteCardSkeletonGrid count={viewMode === "list" ? 4 : 8} />
                </div>
            )}

            {/* Empty State */}
            {!isLoading && displayNotes?.length === 0 && (
                <div className="text-center py-20 bg-background rounded-lg border border-dashed">
                    <div className="text-muted-foreground">
                        {searchQuery
                            ? "No matching notes found"
                            : activeTag
                                ? `No notes with tag "${activeTag}"`
                                : isArchive
                                    ? "No archived notes yet"
                                    : "Your notes will appear here. Click + to create one."
                        }
                    </div>
                </div>
            )}

            {/* Pinned Notes Section */}
            {!isLoading && pinnedNotes.length > 0 && (
                <div className="mb-6">
                    <div className="flex items-center gap-2 mb-3 text-muted-foreground">
                        <Pin className="h-4 w-4 rotate-45" />
                        <span className="text-sm font-medium uppercase tracking-wide">Pinned</span>
                    </div>
                    {renderNotes(pinnedNotes)}
                </div>
            )}

            {/* Other Notes Section */}
            {!isLoading && unpinnedNotes.length > 0 && (
                <div>
                    {pinnedNotes.length > 0 && (
                        <div className="flex items-center gap-2 mb-3 text-muted-foreground border-t pt-4">
                            <span className="text-sm font-medium uppercase tracking-wide">Others</span>
                        </div>
                    )}
                    {renderNotes(unpinnedNotes)}
                </div>
            )}

            {/* Floating Action Button (Mobile only) */}
            {!isArchive && (
                <CreateNoteDialog
                    trigger={
                        <Button
                            size="icon"
                            className="fixed bottom-6 right-6 h-14 w-14 rounded-full shadow-lg md:hidden z-50 hover:scale-105 transition-transform"
                        >
                            <Plus className="h-6 w-6" />
                        </Button>
                    }
                />
            )}
        </div>
    );
}

