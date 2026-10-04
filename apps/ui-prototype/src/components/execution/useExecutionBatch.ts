import { useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  reviewBatch,
  validBatchReview,
  type BatchReview,
} from "../../execution-batch";
import type {
  ExecutionBatchAction,
  ExecutionBatchRequest,
  ExecutionStatus,
  ExecutionView,
} from "../../execution-types";
import { batchReply, batchReceiptText } from "../../execution-batch-receipt";

export function useExecutionBatch(
  runtime: ExecutionView,
  selected: readonly string[],
  available: boolean,
  filterKey: string,
  onBatch: (
    request: ExecutionBatchRequest,
  ) => Promise<ExecutionStatus | undefined>,
  onEscapeCancel: (kind: ExecutionBatchAction["kind"]) => void,
) {
  const [review, setReview] = useState<BatchReview | null>(null);
  const [problem, setProblem] = useState("");
  const [receipt, setReceipt] = useState("");
  const ownSerial = useRef<string | null>(null);
  const ticket = useRef(0);
  const reviewNow = useRef<BatchReview | null>(null);
  const mounted = useRef(true);
  const latest = useRef({
    runtime,
    selected,
    available,
    filterKey,
    onBatch,
    onEscapeCancel,
  });
  latest.current = {
    runtime,
    selected,
    available,
    filterKey,
    onBatch,
    onEscapeCancel,
  };
  const reviewFilter = useRef("");
  const context = JSON.stringify([
    runtime.hostId,
    runtime.catalog.layout,
    runtime.catalog.projectId,
    runtime.sessionId,
  ]);
  const receiptContext = useRef(context);
  useLayoutEffect(() => {
    if (receiptContext.current !== context) {
      receiptContext.current = context;
      ticket.current++;
      ownSerial.current = null;
      setReceipt("");
    }
    if (
      review &&
      (!available ||
        reviewFilter.current !== filterKey ||
        !validBatchReview(review, runtime, selected))
    ) {
      reviewNow.current = null;
      setReview(null);
      setProblem("操作上下文已变化，原批量审阅已取消，请重新核对");
    }
  }, [review, runtime, selected, available, filterKey, context]);
  useEffect(() => {
    mounted.current = true;
    const cancel = () => {
      reviewNow.current = null;
      setReview(null);
    };
    const escape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && reviewNow.current) {
        const kind = reviewNow.current.request.action.kind;
        cancel();
        latest.current.onEscapeCancel(kind);
        e.preventDefault();
        e.stopPropagation();
      }
    };
    const hide = () => {
      if (document.hidden) cancel();
    };
    window.addEventListener("blur", cancel);
    window.addEventListener("keydown", escape, true);
    document.addEventListener("visibilitychange", hide);
    return () => {
      mounted.current = false;
      ticket.current++;
      reviewNow.current = null;
      window.removeEventListener("blur", cancel);
      window.removeEventListener("keydown", escape, true);
      document.removeEventListener("visibilitychange", hide);
    };
  }, []);
  useEffect(() => {
    if (ownSerial.current && runtime.record?.serial === ownSerial.current) {
      const record = runtime.record;
      setReceipt(batchReceiptText(record));
    }
  }, [runtime.record]);
  return {
    review,
    problem,
    receipt,
    cancel() {
      reviewNow.current = null;
      setReview(null);
      setProblem("");
    },
    begin(kind: ExecutionBatchAction["kind"]) {
      if (!latest.current.available) return;
      try {
        const value = reviewBatch(
          latest.current.runtime,
          latest.current.selected,
          kind,
        );
        reviewNow.current = value;
        setReview(value);
        reviewFilter.current = latest.current.filterKey;
        setProblem("");
      } catch (e) {
        setProblem(e instanceof Error ? e.message : String(e));
      }
    },
    async confirm() {
      const now = latest.current;
      const value = reviewNow.current;
      reviewNow.current = null;
      setReview(null);
      if (
        !value ||
        !now.available ||
        now.filterKey !== reviewFilter.current ||
        !validBatchReview(value, now.runtime, now.selected)
      )
        return;
      const serial = now.runtime.record?.serial;
      const currentTicket = ++ticket.current;
      ownSerial.current = null;
      setReceipt("正在提交批量操作，请核对原回执。");
      let result: ExecutionStatus | undefined;
      try {
        result = await now.onBatch(value.request);
      } catch (e) {
        if (mounted.current && ticket.current === currentTicket)
          setReceipt(
            `批量操作结果不可用，请刷新核对，不重复发送：${e instanceof Error ? e.message : String(e)}`,
          );
        return;
      }
      if (
        !mounted.current ||
        ticket.current !== currentTicket ||
        latest.current.runtime.hostId !== value.request.hostId ||
        latest.current.runtime.sessionId !== value.session ||
        latest.current.runtime.catalog.layout !== value.layout ||
        latest.current.runtime.catalog.projectId !== value.project
      )
        return;
      const record = batchReply(value, serial, result);
      if (!record) {
        setReceipt("批量操作未获得匹配回执；请刷新核对，不重复发送。");
        return;
      }
      ownSerial.current = record.serial;
      setReceipt(batchReceiptText(record));
    },
  };
}
