import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { getMutationQueue, clearMutationFromQueue } from './db';
import { api } from './api';
import { toast } from 'sonner';

export async function processQueue(queryClient: any) {
    if (!navigator.onLine) return;

    const queue = await getMutationQueue();

    if (queue.length === 0) return;

    console.log(`Processing ${queue.length} offline mutations...`);
    const toastId = toast.loading('Syncing offline changes...');

    for (const mutation of queue) {
        try {
            if (mutation.type === 'POST') {
                await api.post(mutation.endpoint, mutation.body);
            } else if (mutation.type === 'PATCH') {
                await api.patch(mutation.endpoint, mutation.body);
            } else if (mutation.type === 'DELETE') {
                await api.delete(mutation.endpoint);
            }

            // If we reach here, the request was successful
            await clearMutationFromQueue(mutation.id);

        } catch (error) {
            console.error('Failed to sync mutation:', mutation, error);
            // Decide if we should keep it in queue or remove it.
            // For now, if it's a 4xx error (client error), maybe remove it?
            // If 5xx or network, keep it.
            // Simple strategy: keep it until successful.
        }
    }

    // Refetch data to ensure consistency
    queryClient.invalidateQueries({ queryKey: ['notes'] });
    queryClient.invalidateQueries({ queryKey: ['tags'] });

    toast.dismiss(toastId);
    toast.success('Sync complete');
}

export function useSync() {
    const queryClient = useQueryClient();

    useEffect(() => {
        // Process queue on mount
        processQueue(queryClient);

        // Listen for online status
        const handleOnline = () => {
            console.log('App is back online, syncing...');
            processQueue(queryClient);
        };

        window.addEventListener('online', handleOnline);

        return () => {
            window.removeEventListener('online', handleOnline);
        };
    }, [queryClient]);
}
