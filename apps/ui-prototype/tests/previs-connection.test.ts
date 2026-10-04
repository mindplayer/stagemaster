import { test } from "node:test";
import assert from "node:assert/strict";
import {
  observePrevisConnection,
  previsConnectionView,
} from "../src/components/stage/previs-connection.ts";

function observer() {
  const retained: {
    name: string;
    listener: EventListenerOrEventListenerObject;
  }[] = [];
  class Recorder extends EventTarget {
    override addEventListener(
      name: string,
      listener: EventListenerOrEventListenerObject | null,
      options?: AddEventListenerOptions | boolean,
    ) {
      if (listener) retained.push({ name, listener });
      super.addEventListener(name, listener, options);
    }
  }
  const target = new Recorder();
  const reports: ReturnType<typeof previsConnectionView>[] = [];
  let invalidations = 0;
  const stop = observePrevisConnection(
    target as unknown as Parameters<typeof observePrevisConnection>[0],
    (view) => reports.push(view),
    () => invalidations++,
  );
  return {
    target,
    reports,
    stop,
    invalidations: () => invalidations,
    late(name: string) {
      for (const { name: kind, listener } of retained) {
        if (kind !== name) continue;
        const event = Object.assign(new Event(name), {
          data: { messageStreamerList: { ids: [] } },
        });
        if (typeof listener === "function") listener.call(target, event);
        else listener.handleEvent(event);
      }
    },
    emit(name: string, ids: string[] = []) {
      target.dispatchEvent(
        Object.assign(new Event(name), {
          data: { messageStreamerList: { ids } },
        }),
      );
    },
  };
}

test("等待迟到渲染保持可解释；发现与播放分别确认，不开放无效播放按钮", () => {
  const state = observer();
  for (let attempt = 0; attempt < 6; attempt++)
    state.emit("streamerListMessage");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("waiting"));
  assert.equal(state.invalidations(), 0);
  state.emit("streamerListMessage", ["StageMaster"]);
  assert.deepEqual(state.reports.at(-1), previsConnectionView("connecting"));
  state.emit("playStream");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("playing"));
  state.emit("streamerListMessage");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("playing"));
  state.stop();
});

test("仅真实播放手势被拒绝时允许点击播放，迟到列表不能覆盖手势或错误", () => {
  const state = observer();
  state.emit("playStreamRejected");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("gesture"));
  state.emit("streamerListMessage", ["StageMaster"]);
  assert.equal(state.reports.length, 1);
  state.emit("playStreamError");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("playError"));
  state.emit("streamerListMessage");
  assert.equal(state.reports.length, 2);
  assert.equal(state.invalidations(), 2);
  state.stop();
});

test("断开与失败使编辑连接代次失效，实际再次播放才恢复", () => {
  const state = observer();
  state.emit("playStream");
  state.emit("webRtcDisconnected");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("disconnected"));
  state.emit("webRtcFailed");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("failed"));
  assert.equal(state.invalidations(), 2);
  state.emit("playStream");
  assert.deepEqual(state.reports.at(-1), previsConnectionView("playing"));
  state.stop();
});

test("卸载移除全部旧监听，新观察者不受旧清理与事件影响", () => {
  const state = observer();
  state.stop();
  state.late("playStream");
  state.late("webRtcFailed");
  state.late("streamerListMessage");
  assert.equal(state.reports.length, 0);
  assert.equal(state.invalidations(), 0);
  let nextReports = 0;
  const nextStop = observePrevisConnection(
    state.target as unknown as Parameters<typeof observePrevisConnection>[0],
    () => nextReports++,
    () =>
      assert.fail("stopped listener must not invalidate another connection"),
  );
  state.stop();
  state.emit("playStream");
  assert.equal(nextReports, 1);
  assert.equal(state.reports.length, 0);
  nextStop();
  state.emit("webRtcFailed");
  state.emit("streamerListMessage");
  assert.equal(nextReports, 1);
  assert.equal(state.invalidations(), 0);
});
