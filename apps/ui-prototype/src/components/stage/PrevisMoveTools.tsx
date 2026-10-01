export function PrevisMoveTools({
  moving,
  vertical,
  disabled,
  onAction,
}: {
  moving: boolean;
  vertical: boolean;
  disabled: boolean;
  onAction(action: string): void;
}) {
  if (!moving) return null;
  return (
    <span role="group" aria-label="三维移动方向">
      <button
        disabled={disabled}
        aria-pressed={!vertical}
        onClick={() => onAction("moveHorizontal")}
      >
        水平
      </button>
      <button
        disabled={disabled}
        aria-pressed={vertical}
        onClick={() => onAction("moveVertical")}
      >
        升降
      </button>
    </span>
  );
}
