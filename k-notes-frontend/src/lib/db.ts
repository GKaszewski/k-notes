import { type DBSchema, openDB } from 'idb';

interface NotesDB extends DBSchema {
    notes: {
        key: string;
        value: any;
    };
    mutation_queue: {
        key: number;
        value: {
            id: number;
            type: 'POST' | 'PATCH' | 'DELETE';
            endpoint: string;
            body?: any;
            timestamp: number;
        };
        indexes: { 'by-timestamp': number };
    };
}

const DB_NAME = 'k-notes-db';
const DB_VERSION = 1;

let dbPromise: Promise<import('idb').IDBPDatabase<NotesDB>> | null = null;

export function getDb() {
    if (!dbPromise) {
        dbPromise = openDB<NotesDB>(DB_NAME, DB_VERSION, {
            upgrade(db) {
                // Store for caching notes (optional, if we use manual caching)
                if (!db.objectStoreNames.contains('notes')) {
                    db.createObjectStore('notes', { keyPath: 'id' });
                }

                // Store for offline mutations
                if (!db.objectStoreNames.contains('mutation_queue')) {
                    const store = db.createObjectStore('mutation_queue', {
                        keyPath: 'id',
                        autoIncrement: true,
                    });
                    store.createIndex('by-timestamp', 'timestamp');
                }
            },
        }).catch((err: any) => {
            dbPromise = null; // Reset promise on error so we can retry
            throw err;
        });
    }
    return dbPromise;
}

export type MutationRequest = {
    type: 'POST' | 'PATCH' | 'DELETE';
    endpoint: string;
    body?: any;
};

export async function addToMutationQueue(mutation: MutationRequest) {
    const db = await getDb();
    await db.add('mutation_queue', {
        ...mutation,
        timestamp: Date.now(),
    } as any);
}

export async function getMutationQueue() {
    const db = await getDb();
    return db.getAllFromIndex('mutation_queue', 'by-timestamp');
}

export async function clearMutationFromQueue(id: number) {
    const db = await getDb();
    await db.delete('mutation_queue', id);
}
