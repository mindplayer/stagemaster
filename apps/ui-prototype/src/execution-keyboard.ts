export type ExecutionKeyCommand = "next" | "pause" | "resume" | "stop" | "exit";

export interface ExecutionKeyInput {
  code: string;
  repeat: boolean;
  isComposing: boolean;
  keyCode?: number;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}

export interface ExecutionKeyState {
  busy: boolean;
  ready: boolean;
  canNext: boolean;
  status: "idle" | "running" | "paused" | "finished";
}

const codes = new Set(["Space", "KeyP", "KeyS", "Escape"]);

/** Focus ownership belongs to the UI. This gate never queues a command. */
export class ExecutionKeyGate {
  private held = new Set<string>();

  reset() {
    this.held.clear();
  }

  release(code: string) {
    this.held.delete(code);
  }

  press(
    input: ExecutionKeyInput,
    state: ExecutionKeyState,
  ): { handled: boolean; command?: ExecutionKeyCommand } {
    if (!codes.has(input.code)) return { handled: false };
    const held = this.held.has(input.code);
    // Even rejected presses require a release; dropping a modifier while holding
    // a key must not unexpectedly execute a new command.
    this.held.add(input.code);
    if (
      input.isComposing ||
      input.keyCode === 229 ||
      input.altKey ||
      input.ctrlKey ||
      input.metaKey ||
      input.shiftKey
    )
      return { handled: false };
    if (held || input.repeat) return { handled: true };
    if (input.code === "Escape") return { handled: true, command: "exit" };
    if (state.busy || !state.ready) return { handled: true };
    if (input.code === "Space" && state.canNext)
      return { handled: true, command: "next" };
    if (input.code === "KeyP") {
      if (state.status === "running")
        return { handled: true, command: "pause" };
      if (state.status === "paused")
        return { handled: true, command: "resume" };
    }
    if (input.code === "KeyS" && state.status !== "idle")
      return { handled: true, command: "stop" };
    return { handled: true };
  }
}
