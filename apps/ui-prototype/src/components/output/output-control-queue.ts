import type {
  OutputControlRequest,
  OutputControlSnapshot,
  OutputIntent,
} from "../../output-control-types";
type Port = (request: OutputControlRequest) => Promise<OutputControlSnapshot>;
export interface OutputControlView {
  actual: OutputControlSnapshot | null;
  desired: OutputIntent | null;
  working: boolean;
  recovering: boolean;
  error: string;
}
/** One in-flight write and one merged intent; a read never overwrites newer input. */
export class OutputControlQueue {
  private actual: OutputControlSnapshot | null = null;
  private desired: OutputIntent | null = null;
  private pending: OutputIntent | null = null;
  private working = false;
  private reading = false;
  private recovering = false;
  private revision = 0;
  private disposed = false;
  private error = "";
  private port: Port;
  private publish: (view: OutputControlView) => void;
  constructor(port: Port, publish: (view: OutputControlView) => void) {
    this.port = port;
    this.publish = publish;
  }
  private emit() {
    if (!this.disposed)
      this.publish({
        actual: this.actual,
        desired: this.desired,
        working: this.working,
        recovering: this.recovering,
        error: this.error,
      });
  }
  dispose() {
    this.disposed = true;
    this.pending = null;
  }
  async poll() {
    if (this.disposed || this.working || this.reading) return;
    this.reading = true;
    const revision = this.revision;
    try {
      const value = await this.port({ kind: "snapshot" });
      if (this.disposed || revision !== this.revision) return;
      this.actual = value;
      this.desired = value;
      // A successful status read does not hide a failed write.
      this.emit();
    } catch (reason) {
      if (!this.disposed && revision === this.revision) {
        this.error = reason instanceof Error ? reason.message : String(reason);
        this.emit();
      }
    } finally {
      this.reading = false;
    }
  }
  set(patch: Partial<OutputIntent>) {
    if (this.disposed || this.recovering || !this.actual || !this.desired)
      return;
    const intent = {
      percent: patch.percent ?? this.desired.percent,
      blackout: patch.blackout ?? this.desired.blackout,
    };
    if (
      !Number.isInteger(intent.percent) ||
      intent.percent < 0 ||
      intent.percent > 100
    )
      return;
    this.desired = intent;
    this.pending = intent;
    this.error = "";
    this.revision++;
    this.emit();
    void this.drain();
  }
  private async drain() {
    if (this.working || this.disposed) return;
    this.working = true;
    this.emit();
    try {
      while (this.pending && this.actual && !this.disposed) {
        const intent = this.pending;
        this.pending = null;
        const reply = await this.port({
          kind: "set",
          epoch: this.actual.epoch,
          serial: this.actual.serial + 1,
          ...intent,
        });
        if (this.disposed) return;
        this.actual = reply;
        if (!this.pending) this.desired = reply;
        this.emit();
      }
    } catch (reason) {
      this.pending = null;
      this.recovering = true;
      this.error = reason instanceof Error ? reason.message : String(reason);
      this.emit();
      // A failed transport may still have committed. Reconcile, never replay automatically.
      try {
        this.actual = await this.port({ kind: "snapshot" });
      } catch {
        /* Preserve last receipt. */
      }
      this.desired = this.actual;
    } finally {
      this.working = false;
      this.recovering = false;
      this.emit();
    }
  }
}
