import type { AudioCommand } from "./audio-types";

/** One serialized user action. A rejected seek must never be followed by play. */
export async function runAudioCommands(
  commands: AudioCommand[],
  current: () => boolean,
  send: (command: AudioCommand) => Promise<boolean>,
): Promise<boolean> {
  for (const command of commands) {
    if (!current() || !(await send(command))) return false;
  }
  return current();
}
