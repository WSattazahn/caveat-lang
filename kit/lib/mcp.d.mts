// CLI-private MCP transport. These hooks are not package exports or tool options.
import type { Readable, Writable } from 'node:stream';
import type { spawn } from 'node:child_process';

export declare const MCP_PROTOCOL_VERSION: '2025-11-25';
export declare const MCP_STATELESS_PROTOCOL_VERSION: '2026-07-28';
export declare const MCP_SUPPORTED_VERSIONS: readonly ['2026-07-28', '2025-11-25'];
export interface McpTool {
  name: string;
  description: string;
  inputSchema: { type: 'object'; additionalProperties: false; properties: Record<string, unknown>; required: string[] };
  annotations: { readOnlyHint: true; destructiveHint: false; idempotentHint: true; openWorldHint: false };
}
export declare const MCP_TOOLS: ReadonlyArray<McpTool>;
export interface McpOptions {
  runtimeDirectory?: string;
  input?: Readable;
  output?: Writable;
  /** Internal fault-injection seam. Production always uses Node's spawn. */
  spawnWorker?: typeof spawn;
  /** Internal test timeout; maximum 10,000 ms. */
  timeoutMs?: number;
}
export declare function serveMcp(options?: McpOptions): Promise<number>;
