import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { CubeIcon } from "@phosphor-icons/react";
import type {
  ApplicationHost,
  SceneView,
  FixtureView,
} from "../../application-host";
import type { PrevisInteractions } from "../../previs-types";
import { PrevisPanel } from "./PrevisPanel";
import "./shared-previs.css";

export interface SharedPrevisHandle {
  openPlayback(flushDrafts?: boolean): void;
}

/** The only renderer view in a workbench; workspace navigation never remounts it. */
export const SharedPrevis = forwardRef<
  SharedPrevisHandle,
  {
    host: ApplicationHost;
    scenes: SceneView[];
    limitedFixtures?: FixtureView[];
    currentScene?: SceneView;
    contextKey: string;
    allowPlacement: boolean;
    placementLocked?: boolean;
    busy: boolean;
    fixed?: boolean;
    viewControls?: ReactNode;
    transport?: ReactNode;
    onReveal?(): void;
    onVisibilityChange?(visible: boolean): void;
    generation(): number;
    run(work: () => Promise<void>, flushFirst?: boolean): Promise<boolean>;
  } & PrevisInteractions
>(function SharedPrevis(
  { contextKey, fixed = false, viewControls, onReveal, ...props },
  ref,
) {
  const [visible, setVisible] = useState(fixed);
  const [followCurrent, setFollowCurrent] = useState(false);
  const currentContext = useRef(contextKey);
  const contextChanged = currentContext.current !== contextKey;
  useEffect(
    () => props.onVisibilityChange?.(visible),
    [visible, props.onVisibilityChange],
  );
  useEffect(() => {
    currentContext.current = contextKey;
    setFollowCurrent(false);
  }, [contextKey]);

  function open(playback = false, flushDrafts = true) {
    void props.run(async () => {
      if (playback) {
        await props.host.previs({
          kind: "source",
          generation: props.generation(),
          source: { kind: "playback" },
        });
        setFollowCurrent(false);
      }
      const status = await props.host.previs({ kind: "status" });
      if (!status.enabled) await props.host.previs({ kind: "enable" });
      setVisible(true);
      onReveal?.();
    }, flushDrafts);
  }
  useImperativeHandle(ref, () => ({
    openPlayback: (flushDrafts) => open(true, flushDrafts),
  }));

  return (
    <section
      className={`wb-shared-previs${visible ? " expanded" : ""}`}
      aria-label="公共三维预演"
    >
      <header>
        {viewControls}
        {fixed ? (
          viewControls ? null : (
            <strong>三维舞台</strong>
          )
        ) : (
          <button
            disabled={props.busy || props.host.kind !== "desktop"}
            aria-expanded={visible}
            onClick={() =>
              visible ? void props.run(async () => setVisible(false)) : open()
            }
          >
            <CubeIcon />
            {visible ? "收起三维" : "三维预演"}
          </button>
        )}
      </header>
      {visible && (
        <>
          <PrevisPanel
            {...props}
            contextKey={contextKey}
            followCurrent={followCurrent && !contextChanged}
            onFollowCurrent={setFollowCurrent}
          />
          {props.transport}
        </>
      )}
    </section>
  );
});
