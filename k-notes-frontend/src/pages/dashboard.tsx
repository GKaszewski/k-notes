import { useState } from "react";
import { useNotes, useSearchNotes } from "@/hooks/use-notes";
import { CreateNoteDialog } from "@/components/create-note-dialog";
import { NoteCard } from "@/components/note-card";
import { Input } from "@/components/ui/input";
import { Search } from "lucide-react";
import { useLocation } from "react-router-dom";

export default function DashboardPage() {
    const location = useLocation();
    const isArchive = location.pathname === "/archive";
    
    // Search state
    const [searchQuery, setSearchQuery] = useState("");
    
    // Fetch normal notes only if not searching
    const { data: notes, isLoading: notesLoading } = useNotes(searchQuery ? undefined : { archived: isArchive });
    
    // Fetch search results if searching
    const { data: searchResults, isLoading: searchLoading } = useSearchNotes(searchQuery);

    const displayNotes = searchQuery ? searchResults : notes;
    const isLoading = searchQuery ? searchLoading : notesLoading;

    return (
        <div className="max-w-7xl mx-auto">
            {/* Action Bar */}
            <div className="flex flex-col md:flex-row gap-4 justify-between items-center mb-6">
                 <div className="relative w-full md:w-96">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input 
                        placeholder="Search your notes..." 
                        className="pl-9 w-full bg-background"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                    />
                </div>
                
                {!isArchive && <CreateNoteDialog />}
            </div>

            {/* Title */}
            <h1 className="text-2xl font-bold mb-4 hidden">
                {isArchive ? "Archive" : "Notes"}
            </h1>

            {/* Loading State */}
            {isLoading && (
                <div className="text-center py-12 text-muted-foreground animate-pulse">
                    Loading your ideas...
                </div>
            )}

            {/* Empty State */}
            {!isLoading && displayNotes?.length === 0 && (
                <div className="text-center py-20 bg-background rounded-lg border border-dashed">
                    <div className="text-muted-foreground">
                        {searchQuery 
                            ? "No matching notes found" 
                            : isArchive 
                                ? "No archived notes yet"
                                : "Your notes will appear here. Click + to create one."
                        }
                    </div>
                </div>
            )}

            {/* Notes Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 items-start">
                {/* Pinned Notes First (if not searching and not archive) */}
                {!searchQuery && !isArchive && displayNotes?.filter((n: any) => n.is_pinned).map((note: any) => (
                        <NoteCard key={note.id} note={note} />
                ))}
                
                {/* Other Notes */}
                {displayNotes?.filter((n: any) => searchQuery || isArchive || !n.is_pinned).map((note: any) => (
                    <NoteCard key={note.id} note={note} />
                ))}
            </div>
        </div>
    );
}
