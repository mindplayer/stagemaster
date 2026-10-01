export class AudioDraftError extends Error {
  readonly field: string;
  constructor(message: string, field: string) {
    super(message);
    this.field = field;
  }
}
