// Types for starter-node.mjs: the starter's store in a Node directory.
import type { StarterStore } from './starter.mjs';

export interface FileStore extends StarterStore {
  readonly durable: true;
  readonly directory: string;
}

/**
 * checkpoint.json, replaced through a synced temporary file and rename, and
 * archive.jsonl, appended and synced. Opening it cuts off a torn last line.
 */
export declare function fileStore(directory: string): Promise<FileStore>;
