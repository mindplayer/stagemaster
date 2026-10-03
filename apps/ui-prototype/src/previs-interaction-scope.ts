/** Invalidates delayed placement work when a viewer changes editing context. */
export class PrevisInteractionScope {
  private key = "";
  private allowed = false;
  private revision = 0;

  update(key: string, allowed: boolean) {
    if (key !== this.key || allowed !== this.allowed) {
      this.key = key;
      this.allowed = allowed;
      this.revision++;
    }
  }

  invalidate() {
    this.revision++;
  }

  capture() {
    const revision = this.revision;
    return () => this.allowed && revision === this.revision;
  }
}
