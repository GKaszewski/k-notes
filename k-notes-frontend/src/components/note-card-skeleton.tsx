import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

export function NoteCardSkeleton() {
    return (
        <Card className="relative">
            <CardHeader className="pb-2">
                <div className="flex justify-between items-start">
                    <Skeleton className="h-5 w-3/4" />
                    <Skeleton className="h-4 w-4 rounded-full" />
                </div>
                <Skeleton className="h-3 w-24 mt-1" />
            </CardHeader>
            <CardContent className="pb-2 space-y-2">
                <Skeleton className="h-3 w-full" />
                <Skeleton className="h-3 w-full" />
                <Skeleton className="h-3 w-4/5" />
            </CardContent>
            <CardFooter className="flex flex-col items-start gap-2 pt-2">
                <div className="flex flex-wrap gap-1">
                    <Skeleton className="h-5 w-12 rounded-full" />
                    <Skeleton className="h-5 w-16 rounded-full" />
                </div>
                <div className="flex justify-end w-full gap-1">
                    <Skeleton className="h-8 w-8 rounded" />
                    <Skeleton className="h-8 w-8 rounded" />
                    <Skeleton className="h-8 w-8 rounded" />
                </div>
            </CardFooter>
        </Card>
    );
}

export function NoteCardSkeletonGrid({ count = 8 }: { count?: number }) {
    return (
        <>
            {Array.from({ length: count }).map((_, i) => (
                <NoteCardSkeleton key={i} />
            ))}
        </>
    );
}
