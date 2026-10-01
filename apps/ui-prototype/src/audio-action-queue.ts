import type { AudioCommand } from "./audio-types";

type Runner = (commands: AudioCommand[]) => Promise<boolean>;
interface Action {
  commands: AudioCommand[];
  context: string;
  run: Runner;
  promise: Promise<boolean>;
  resolve(value: boolean): void;
  reject(error: unknown): void;
}
const coalescible = (commands: AudioCommand[]) =>
  commands.length === 1 && ["seek", "volume"].includes(commands[0].kind)
    ? commands[0].kind
    : null;

/** Ordered controls with bounded pending batches and no coalescing across transport barriers. */
export class AudioActionQueue {
  private waiting: Action[] = [];
  private running = false;
  private revision = 0;

  observation(): number | null {
    return this.running || this.waiting.length ? null : this.revision;
  }
  acceptsObservation(revision: number): boolean {
    return this.observation() === revision;
  }
  invalidate(): void {
    this.revision++;
    for (const action of this.waiting.splice(0)) action.resolve(false);
  }
  enqueue(
    commands: AudioCommand[],
    context: string,
    run: Runner,
  ): Promise<boolean> {
    this.revision++;
    const previous = this.waiting.at(-1);
    const kind = coalescible(commands);
    if (
      previous &&
      kind &&
      previous.context === context &&
      coalescible(previous.commands) === kind
    ) {
      previous.commands = commands;
      previous.run = run;
      return previous.promise;
    }
    if (this.waiting.length >= 32)
      return Promise.reject(new Error("音乐操作过密，请等待当前操作完成"));
    let resolve!: Action["resolve"], reject!: Action["reject"];
    const promise = new Promise<boolean>((yes, no) => {
      resolve = yes;
      reject = no;
    });
    this.waiting.push({ commands, context, run, promise, resolve, reject });
    void this.drain();
    return promise;
  }
  private async drain(): Promise<void> {
    if (this.running) return;
    this.running = true;
    try {
      while (this.waiting.length) {
        const action = this.waiting.shift()!;
        try {
          action.resolve(await action.run(action.commands));
        } catch (error) {
          action.reject(error);
        }
      }
    } finally {
      this.running = false;
    }
  }
}
