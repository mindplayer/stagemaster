import type {
  RiggingCommand,
  RiggingPreviewPort,
  RiggingProjection,
} from "../../rigging-preview-types";

export type RiggingPreviewResult = {
  key: string;
  projection?: RiggingProjection;
  error?: string;
};
type Job = {
  serial: number;
  key: string;
  generation: number;
  command: RiggingCommand;
};
/** One active request and one latest intent; cancellation never starts a parallel native job. */
export class RiggingPreviewQueue {
  private serial = 0;
  private active = false;
  private next: Job | null = null;
  private request: RiggingPreviewPort;
  private deliver: (value: RiggingPreviewResult) => void;
  constructor(
    request: RiggingPreviewPort,
    deliver: (value: RiggingPreviewResult) => void,
  ) {
    this.request = request;
    this.deliver = deliver;
  }
  submit(key: string, generation: number, command: RiggingCommand) {
    this.next = { serial: ++this.serial, key, generation, command };
    void this.pump();
  }
  cancel() {
    this.serial++;
    this.next = null;
  }
  private async pump() {
    if (this.active || !this.next) return;
    const job = this.next;
    this.next = null;
    this.active = true;
    try {
      const projection = await this.request(job.generation, job.command);
      if (projection.generation !== job.generation)
        throw new Error("工程代次不匹配，请重新预览");
      if (this.serial === job.serial)
        this.deliver({ key: job.key, projection });
    } catch (e) {
      if (this.serial === job.serial)
        this.deliver({
          key: job.key,
          error: e instanceof Error ? e.message : String(e),
        });
    } finally {
      this.active = false;
      void this.pump();
    }
  }
}
