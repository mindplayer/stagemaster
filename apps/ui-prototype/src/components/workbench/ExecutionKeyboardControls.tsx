import { useEffect, useId, useRef, useState } from "react";
import {
  ExecutionKeyGate,
  type ExecutionKeyCommand,
  type ExecutionKeyState,
} from "../../execution-keyboard";
import "./execution-keyboard.css";

/** Remount for a new list/player epoch; only the focused pad accepts commands. */
export function ExecutionKeyboardControls({
  visible,
  state,
  onCommand,
}: {
  visible: boolean;
  state: ExecutionKeyState;
  onCommand(command: Exclude<ExecutionKeyCommand, "exit">): Promise<void>;
}) {
  const [armed, setArmed] = useState(false);
  const gate = useRef(new ExecutionKeyGate());
  const pending = useRef(false);
  const pad = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const hintId = useId();
  const enabled = visible && state.ready;
  const active = armed && enabled;
  function disarm() {
    gate.current.reset();
    setArmed(false);
  }
  useEffect(() => {
    if (!enabled) disarm();
  }, [enabled]);
  useEffect(() => {
    if (active) pad.current?.focus();
  }, [active]);
  useEffect(() => {
    const hidden = () => {
      if (document.hidden) disarm();
    };
    window.addEventListener("blur", disarm);
    document.addEventListener("visibilitychange", hidden);
    return () => {
      window.removeEventListener("blur", disarm);
      document.removeEventListener("visibilitychange", hidden);
    };
  }, []);

  return (
    <section className="execution-keyboard" aria-label="键盘执行">
      <button
        ref={button}
        type="button"
        aria-pressed={active}
        disabled={!enabled || state.busy}
        onClick={() => {
          if (active) disarm();
          else setArmed(true);
        }}
      >
        {active ? "退出键盘执行" : "键盘执行"}
      </button>
      {active ? (
        <div
          ref={pad}
          className="execution-keyboard-pad"
          role="group"
          aria-label="键盘执行区"
          aria-describedby={hintId}
          tabIndex={0}
          onBlur={disarm}
          onKeyUp={(event) => gate.current.release(event.code)}
          onKeyDown={(event) => {
            if (
              !active ||
              document.hidden ||
              !document.hasFocus() ||
              event.target !== event.currentTarget
            )
              return;
            const result = gate.current.press(event.nativeEvent, {
              ...state,
              busy: state.busy || pending.current,
            });
            if (result.handled) {
              event.preventDefault();
              event.stopPropagation();
            }
            if (result.command === "exit") {
              disarm();
              button.current?.focus();
            } else if (result.command) {
              pending.current = true;
              void onCommand(result.command).finally(() => {
                pending.current = false;
              });
            }
          }}
        >
          <strong>键盘执行已开启</strong>
          <span>
            <kbd>空格</kbd> 下一步 · <kbd>P</kbd> 暂停／继续
          </span>
          <span>
            <kbd>S</kbd> 停止 · <kbd>Esc</kbd> 退出键盘
          </span>
          <small id={hintId}>松开按键后可再次执行。离开此区域自动退出。</small>
        </div>
      ) : (
        <small>启用后，空格推进下一步。</small>
      )}
    </section>
  );
}
