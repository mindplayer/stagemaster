/** One observation and at most one waiting user action per connection.
 * Polling never drops a click or overlaps a command; reconnect discards old work.
 */
export class DeviceRuntimeWork {
  private observation: Promise<void> | null = null;
  private active = true;
  commandPending = false;

  async observe(work: () => Promise<void>): Promise<void> {
    if (!this.active || this.observation || this.commandPending) return;
    const result = Promise.resolve().then(work);
    this.observation = result;
    try {
      await result;
    } finally {
      this.observation = null;
    }
  }

  async command(work: () => Promise<void>): Promise<void> {
    if (!this.active || this.commandPending) return;
    this.commandPending = true;
    try {
      // A failed observation must not admit a command based on stale state.
      await this.observation;
      if (this.active) await work();
    } finally {
      this.commandPending = false;
    }
  }

  invalidate(): void {
    this.active = false;
  }
}
