import type { KeyboardEvent } from "react";
/** Workspace shortcuts never consume input controls or nested timeline gestures. */
export function audioWorkspaceShortcuts(props: {
  blocked: boolean;
  playPause(): void;
  addMarker(): void;
}) {
  return (event: KeyboardEvent) => {
    if (
      event.target instanceof HTMLElement &&
      event.target.closest(
        "input,select,textarea,button,[contenteditable=true]",
      )
    )
      return;
    if (props.blocked || event.repeat) return;
    if (event.code === "Space") {
      event.preventDefault();
      props.playPause();
    }
    if (event.key.toLowerCase() === "m" && !event.metaKey && !event.ctrlKey) {
      event.preventDefault();
      props.addMarker();
    }
  };
}
