import { useEffect, useRef, useState } from "react";
import type { ApplicationHost } from "../../application-host";
import type {
  InstallationRequest,
  InstallationView,
} from "../../installation-types";
import { deviceDeadline, deviceError } from "../../device-tools";
import { mergeInstallation } from "../../installation-tools";

export function useInstallation(host: ApplicationHost) {
  const [view, setView] = useState<InstallationView | null>(null);
  const [open, setOpen] = useState(false);
  const [error, setError] = useState("");
  const [communicationError, setCommunicationError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(false);
  const serial = useRef(0);
  const received = useRef(0);
  const current = useRef(view);
  current.current = view;
  function accept(next: InstallationView, order: number) {
    setView((old) => mergeInstallation(old, next));
    if (order >= received.current) {
      received.current = order;
      setCommunicationError("");
    }
  }
  useEffect(() => {
    mounted.current = true;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      const order = ++serial.current;
      try {
        const next = await deviceDeadline(
          host.installation({ kind: "status" }),
        );
        if (!stopped) accept(next, order);
      } catch (reason) {
        if (!stopped && order >= received.current)
          setCommunicationError(deviceError(reason));
      }
      if (!stopped) timer = setTimeout(() => void poll(), 700);
    }
    if (host.kind === "desktop") void poll();
    return () => {
      stopped = true;
      mounted.current = false;
      clearTimeout(timer);
    };
  }, [host]);
  async function perform(work: () => Promise<InstallationView>) {
    if (pending.current) return;
    pending.current = true;
    setBusy(true);
    setError("");
    const order = ++serial.current;
    try {
      const next = await deviceDeadline(work());
      if (mounted.current) accept(next, order);
    } catch (reason) {
      if (mounted.current) setError(deviceError(reason));
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  function request(value: InstallationRequest) {
    return perform(() => host.installation(value));
  }
  function start(generation: number, token: string) {
    setOpen(true);
    return perform(() => {
      const target = current.current?.destination;
      if (!target?.allowed || !target.deviceId)
        throw new Error(target?.reason || "请先连接具有安装权限的设备");
      return host.startInstallation(
        generation,
        token,
        target.epoch,
        target.deviceId,
      );
    });
  }
  return {
    view,
    open,
    setOpen,
    busy,
    error,
    communicationError,
    request,
    start,
    available: host.kind === "desktop",
  };
}
export type InstallationController = ReturnType<typeof useInstallation>;
