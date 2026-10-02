// CLI-internal demo report, with the actual public snapshot/explanation types.
import type { CaveatRuntime, Provenance, RuntimeIdentity, Snapshot } from './session.mjs';
import type { ExplainReport } from './explain.mjs';

export declare const DEMO_SCHEMA: 'caveat-demo/0.1';

export interface AgentDemoStep {
  event: string;
  payload: Record<string, number>;
  snapshot: Snapshot;
  explanation: ExplainReport;
}

export interface AgentDemoReport {
  schema: 'caveat-demo/0.1';
  demo: 'agent';
  inputs: 'illustrative';
  runtime: RuntimeIdentity;
  source: { file: string; sha256: string };
  steps: AgentDemoStep[];
  preservation: {
    commitment: string;
    originalGrounds: Provenance;
    finalGrounds: Provenance;
    unchanged: boolean;
  };
}

export declare function runAgentDemo(runtime: CaveatRuntime): Promise<AgentDemoReport>;
export declare function formatAgentDemo(report: AgentDemoReport): string;
