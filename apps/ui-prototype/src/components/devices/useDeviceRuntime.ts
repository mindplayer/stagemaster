import { useEffect, useRef, useState } from "react";
import type { DeviceSnapshot } from "../../device-types";
import type {
  DeviceProgram,
  DeviceRunAction,
  DeviceRunPort,
  DeviceRunReply,
  DeviceRunRequest,
  DeviceRunView,
  DeviceStep,
} from "../../device-runtime-types";
import {
  acceptRunReply,
  ownsRun,
  programKey,
  runState,
  shouldRenew,
} from "../../device-runtime-tools";
import { deviceError } from "../../device-tools";

// Mounted by DeviceCenter, not by its collapsible body. Never owns the device clock.
export function useDeviceRuntime(
  port: DeviceRunPort | undefined,
  connection: DeviceSnapshot | null,
) {
  const [view, setView] = useState<DeviceRunView | null>(null);
  const [observed, setObserved] = useState<DeviceRunReply | null>(null);
  const [programs, setPrograms] = useState<DeviceProgram[]>([]);
  const [steps, setSteps] = useState<DeviceStep[]>([]);
  const [busy, setBusy] = useState(false);
  const [listing, setListing] = useState("");
  const [error, setError] = useState("");
  const [readError, setReadError] = useState("");
  const [lease, setLease] = useState<string | null>(null);
  const latest = useRef<{
    reply: DeviceRunReply | null;
    view: DeviceRunView | null;
    lease: string | null;
  }>({ reply: null, view: null, lease: null });
  const current = useRef(connection);
  current.current = connection;
  const generation = useRef(0),
    pending = useRef(false),
    cancelList = useRef(false);
  const renewed = useRef("0");
  const known = useRef({ catalog: "", steps: "" });
  const epoch = connection?.epoch;
  function live(token: number, expected: number) {
    return (
      generation.current === token &&
      current.current?.epoch === expected &&
      current.current.phase === "connected"
    );
  }
  async function send(request: DeviceRunRequest, token: number) {
    let next: DeviceRunView;
    try {
      next = await port!(request);
    } catch (e) {
      if (live(token, request.epoch)) setReadError(deviceError(e));
      throw e;
    }
    if (!live(token, request.epoch) || next.epoch !== request.epoch)
      throw new Error("连接已变化，旧回复未应用");
    latest.current.view = next;
    setView(next);
    const accepted = acceptRunReply(latest.current.reply, next, request.epoch);
    if (accepted?.body.kind === "state") {
      latest.current.reply = accepted;
      setObserved(accepted);
      if (!ownsRun(accepted.body.state, latest.current.lease)) {
        latest.current.lease = null;
        setLease(null);
      }
    }
    if (next.reply?.body.kind === "state" && next.reply.body.error)
      throw new Error(next.reply.body.error);
    return next;
  }
  async function pages(
    kind: "catalog" | "step",
    token: number,
    expected: number,
  ) {
    const base = latest.current.reply;
    if (!base) return;
    cancelList.current = false;
    const count = kind === "catalog" ? base.programCount : base.stepCount;
    const items: (DeviceProgram | DeviceStep)[] = [];
    try {
      for (let index = 0; index < count; index++) {
        if (cancelList.current) throw new Error("目录读取已取消，可重新读取");
        setListing(
          `正在读取${kind === "catalog" ? "节目" : "步骤"} ${index + 1}/${count}`,
        );
        const result = await send(
          { kind, epoch: expected, revision: base.revision, index },
          token,
        );
        if (cancelList.current) throw new Error("目录读取已取消，可重新读取");
        const reply = result.reply;
        if (
          !reply ||
          reply.boot !== base.boot ||
          reply.revision !== base.revision
        )
          throw new Error("设备目录已变化，请重新读取");
        if (
          kind === "catalog" &&
          reply.body.kind === "program" &&
          reply.body.program
        )
          items.push(reply.body.program);
        else if (
          kind === "step" &&
          reply.body.kind === "step" &&
          reply.body.step
        )
          items.push(reply.body.step);
        else throw new Error("设备目录回复不完整，请重新读取");
      }
      if (kind === "catalog") setPrograms(items as DeviceProgram[]);
      else setSteps(items as DeviceStep[]);
    } finally {
      if (live(token, expected)) setListing("");
    }
  }
  async function refresh(token: number, expected: number, force = false) {
    let snapshot: DeviceRunView;
    try {
      snapshot = await send({ kind: "snapshot", epoch: expected }, token);
      if (!snapshot.peer) return;
      await send({ kind: "refresh", epoch: expected }, token);
      setReadError("");
    } catch (e) {
      if (live(token, expected)) setReadError(deviceError(e));
      throw e;
    }
    const reply = latest.current.reply,
      state = runState(reply);
    if (!state || !reply) return;
    if (
      shouldRenew(
        state,
        reply.observedMs,
        latest.current.lease,
        renewed.current,
      )
    ) {
      try {
        const next = await send(
          {
            kind: "apply",
            epoch: expected,
            revision: reply.revision,
            action: { kind: "renew" },
          },
          token,
        );
        renewed.current = next.reply?.observedMs ?? reply.observedMs;
      } catch (e) {
        latest.current.lease = null;
        setLease(null);
        throw e;
      }
    }
    const catalog = `${snapshot.peer.session}:${state.package}`;
    const stepKey = `${catalog}:${programKey(state.loaded)}`;
    if (force || known.current.catalog !== catalog) {
      known.current.catalog = catalog;
      setPrograms([]);
      await pages("catalog", token, expected);
    }
    if (force || known.current.steps !== stepKey) {
      known.current.steps = stepKey;
      setSteps([]);
      await pages("step", token, expected);
    }
  }
  useEffect(() => {
    const token = ++generation.current;
    pending.current = false;
    setBusy(false);
    setListing("");
    latest.current = { reply: null, view: null, lease: null };
    setView(null);
    setObserved(null);
    setLease(null);
    setPrograms([]);
    setSteps([]);
    known.current = { catalog: "", steps: "" };
    setReadError("");
    if (!port || epoch == null) return;
    if (connection?.phase !== "connected") {
      void port({ kind: "snapshot", epoch })
        .then((next) => {
          if (generation.current === token && current.current?.epoch === epoch)
            setView({ ...next, peer: null });
        })
        .catch(() => {});
      return () => {
        ++generation.current;
      };
    }
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      if (!live(token, epoch!)) return;
      if (!pending.current) {
        pending.current = true;
        setBusy(true);
        try {
          await refresh(token, epoch!);
          if (live(token, epoch!)) setReadError("");
        } catch (e) {
          if (live(token, epoch!)) setReadError(deviceError(e));
        } finally {
          if (live(token, epoch!)) {
            pending.current = false;
            setBusy(false);
          }
        }
      }
      if (live(token, epoch!)) timer = setTimeout(() => void poll(), 1000);
    }
    void poll();
    return () => {
      ++generation.current;
      clearTimeout(timer);
    };
  }, [port, epoch, connection?.phase]);
  async function perform(
    work: (token: number, expected: number) => Promise<void>,
  ) {
    if (
      !port ||
      pending.current ||
      epoch == null ||
      connection?.phase !== "connected"
    )
      return;
    const token = generation.current;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      await work(token, epoch);
      if (live(token, epoch)) setReadError("");
    } catch (e) {
      if (live(token, epoch)) setError(deviceError(e));
    } finally {
      if (live(token, epoch)) {
        pending.current = false;
        setBusy(false);
      }
    }
  }
  async function action(
    value: DeviceRunAction,
    token: number,
    expected: number,
  ) {
    const revision = latest.current.reply?.revision;
    if (!revision || !latest.current.view?.peer || readError)
      throw new Error("请先读取当前设备状态");
    const result = await send(
      { kind: "apply", epoch: expected, revision, action: value },
      token,
    );
    if (value.kind === "acquire" && result.reply?.body.kind === "state") {
      const next = result.reply.body.state.owner?.lease ?? null;
      latest.current.lease = next;
      setLease(next);
      renewed.current = result.reply.observedMs;
    }
    if (value.kind === "release") {
      latest.current.lease = null;
      setLease(null);
    }
  }
  const valid =
    !!view &&
    connection?.phase === "connected" &&
    view.epoch === epoch &&
    view.connectionEpoch === epoch;
  const state = valid ? runState(observed) : null;
  return {
    view:
      view?.epoch === epoch
        ? connection?.phase === "connected"
          ? view
          : { ...view, peer: null }
        : null,
    state,
    programs,
    programCount: valid ? (observed?.programCount ?? null) : null,
    steps,
    busy,
    listing,
    ready: valid && !readError,
    error: error || readError,
    owned: !!valid && !!view?.peer && ownsRun(state, lease),
    request: (value: DeviceRunAction) =>
      perform(async (token, expected) => {
        await action(value, token, expected);
        await refresh(token, expected);
      }),
    load: (program: DeviceProgram) =>
      perform(async (token, expected) => {
        if (
          programKey(runState(latest.current.reply)?.selected ?? null) !==
          programKey(program.key)
        )
          await action(
            { kind: "select", program: program.key },
            token,
            expected,
          );
        await action({ kind: "load" }, token, expected);
        await refresh(token, expected);
      }),
    reload: () => perform((token, expected) => refresh(token, expected, true)),
    cancel: () => {
      cancelList.current = true;
      setListing("已取消，正在等待当前回复结束");
    },
  };
}
export type DeviceRuntimeController = ReturnType<typeof useDeviceRuntime>;
