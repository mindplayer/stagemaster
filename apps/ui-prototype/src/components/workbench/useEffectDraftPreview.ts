import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
import type { ApplicationHost } from "../../application-host";
import type { EffectHandle } from "./EffectEditor";
import {
  EffectDraftSession,
  effectDraftValue,
  type EffectDraftState,
} from "./effect-draft-session";
export interface EffectAuditionControls extends EffectDraftState {
  toggle(): void;
}

export function useEffectDraftPreview({
  host,
  editor,
  target,
  available,
  canStart,
  openPlayback,
}: {
  host: ApplicationHost;
  editor: RefObject<EffectHandle | null>;
  target?: string;
  available: boolean;
  canStart(): boolean;
  openPlayback(): void;
}) {
  const [state, setState] = useState<EffectDraftState>({
    active: false,
    working: false,
    message: "",
  });
  const [revision, setRevision] = useState(0);
  const live = useRef(true);
  const current = useRef({ target, available, canStart, openPlayback });
  current.current = { target, available, canStart, openPlayback };
  const session = useMemo(
    () =>
      new EffectDraftSession(host, (value) => {
        if (live.current) setState(value);
      }),
    [host],
  );
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
      void session.end();
    };
  }, [session]);
  useEffect(() => {
    return () => {
      void session.end();
    };
  }, [session, target, available]);
  useEffect(() => {
    if (!state.active) return;
    const timer = setTimeout(() => {
      try {
        session.update(
          effectDraftValue(editor.current?.collect(false, true) ?? []),
        );
      } catch (e) {
        session.problem(e);
      }
    }, 250);
    return () => clearTimeout(timer);
  }, [revision, state.active, session, editor]);
  useEffect(() => {
    if (!state.active) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      await session.inspect();
      if (!cancelled) timer = setTimeout(poll, 500);
    };
    timer = setTimeout(poll, 500);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [session, state.active]);
  async function toggle() {
    if (state.active) {
      await session.end();
      return;
    }
    try {
      if (!current.current.available || !current.current.target) return;
      if (!current.current.canStart())
        throw new Error("请先应用或取消其他属性的修改，再开启效果即时预演");
      const captured = current.current.target;
      const value = effectDraftValue(editor.current?.collect(true, true) ?? []);
      if (await session.begin(value)) {
        if (captured === current.current.target && current.current.available)
          current.current.openPlayback();
        else await session.end();
      }
    } catch (e) {
      session.problem(e);
    }
  }
  return {
    controls: { ...state, toggle: () => void toggle() },
    changed: () => setRevision((v) => v + 1),
    end: () => session.end(),
  };
}
