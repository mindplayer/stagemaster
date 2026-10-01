import { useEffect, useRef, useState } from "react";

/** Capture valid drafts once; late completion never writes into an unmounted project. */
export function useReportExport<Receipt extends { path: string | null }>({
  busy,
  capture,
}: {
  busy: boolean;
  capture(): Promise<number | null>;
}) {
  const [working, setWorking] = useState(false);
  const [receipt, setReceipt] = useState<Receipt | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const active = useRef(false),
    mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  async function start(run: (version: number) => Promise<Receipt>) {
    if (active.current || busy) return;
    active.current = true;
    setWorking(true);
    setError("");
    setMessage("");
    setReceipt(null);
    try {
      const version = await capture();
      if (version === null || !mounted.current) return;
      const result = await run(version);
      if (!mounted.current) return;
      if (result.path) setReceipt(result);
      else setMessage("已取消导出");
    } catch (error) {
      if (mounted.current)
        setError(error instanceof Error ? error.message : String(error));
    } finally {
      active.current = false;
      if (mounted.current) setWorking(false);
    }
  }
  return { working, receipt, message, error, start };
}
