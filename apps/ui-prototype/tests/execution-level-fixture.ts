import { manualFixture } from "./execution-manual-fixture.ts";
import type { ExecutionStatus } from "../src/execution-types.ts";
export function levelFixture(): ExecutionStatus {
  const runtime = manualFixture();
  runtime.catalog.sources[0].name = "蓝色逆光";
  runtime.observation.snapshot!.state.owner = {
    sessionId: runtime.sessionId!,
    expiresMs: "60000",
  };
  return { phase: "connected", problem: null, runtime };
}
