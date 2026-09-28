import { useEffect, useState } from "react";
import type { InstallationController } from "./useInstallation";
import type { InstallationTask as Task } from "../../installation-types";
import {
  installationFinished,
  installationLabels,
  resumeInstallationReason,
} from "../../installation-tools";
const size = (n: number) => `${(n / 1024).toFixed(1)} KiB`;
export function InstallationTask({
  task,
  controller,
}: {
  task: Task;
  controller: InstallationController;
}) {
  const [forgetting, setForgetting] = useState(false);
  useEffect(() => setForgetting(false), [task.id]);
  const { view, busy, communicationError, request } = controller;
  const finished = installationFinished(task.phase);
  const resumeReason = view
    ? resumeInstallationReason(task, view.destination)
    : "请先读取设备状态";
  const disabled = busy || !!communicationError;
  return (
    <section className="wb-install-task" aria-label="当前安装任务">
      <div className="wb-install-phase">
        <strong role="status">
          {communicationError
            ? "任务状态不可用"
            : installationLabels[task.phase]}
        </strong>
        <span>{task.deviceName}</span>
      </div>
      <p>
        {task.package.projectName} · {task.package.programs} 个节目
      </p>
      <progress
        max={task.package.bytes}
        value={task.confirmedBytes}
        aria-label="设备确认接收的字节"
      />
      <p className="wb-install-caption">
        {task.phase === "reconnect" || communicationError
          ? "上次确认"
          : "设备已接收"}{" "}
        {size(task.confirmedBytes)} / {size(task.package.bytes)}
      </p>
      {task.problem && (
        <p className="wb-install-error" role="alert">
          {task.problem}
        </p>
      )}
      {task.cancelRequested && !finished && (
        <p>取消请求已保留，等待设备确认；收起面板不会撤销此请求。</p>
      )}
      {task.receipt && (
        <div className="wb-install-receipt">
          <b>设备已确认持久保存</b>
          <span>
            第 {task.receipt.generation} 代 · {size(task.receipt.bytes)}
          </span>
          {task.cancelRequested && <p>设备已完成提交，本次取消未撤销安装。</p>}
        </div>
      )}
      <div className="wb-install-actions">
        {!finished && (
          <button
            disabled={disabled || task.cancelRequested}
            onClick={() => void request({ kind: "cancel", id: task.id })}
          >
            取消安装
          </button>
        )}
        {!finished && !task.running && (
          <button
            className="wb-primary"
            disabled={disabled || !!resumeReason}
            title={resumeReason || undefined}
            onClick={() =>
              void request({
                kind: "resume",
                id: task.id,
                epoch: view!.destination.epoch,
              })
            }
          >
            {task.cancelRequested ? "继续核对取消结果" : "继续安装并核对"}
          </button>
        )}
        {!task.running && (
          <button
            disabled={disabled}
            onClick={() =>
              finished
                ? void request({ kind: "forget", id: task.id })
                : setForgetting(true)
            }
          >
            {finished ? "清除完成记录" : "移除本机任务"}
          </button>
        )}
      </div>
      {!task.running && !finished && resumeReason && (
        <p className="wb-install-caption">{resumeReason}</p>
      )}
      {forgetting && (
        <div
          className="wb-install-confirm"
          role="group"
          aria-label="确认移除本机任务"
        >
          <p>
            这只移除本机任务，不会撤销设备中的安装。设备可能仍有未完成事务或已经提交的节目，后续须重新核对。
          </p>
          <button
            disabled={disabled}
            onClick={() => void request({ kind: "forget", id: task.id })}
          >
            确认移除本机任务
          </button>
          <button onClick={() => setForgetting(false)}>保留任务</button>
        </div>
      )}
      <details>
        <summary>安装校验信息</summary>
        <dl>
          <dt>原定设备</dt>
          <dd>{task.deviceId}</dd>
          <dt>播放包摘要</dt>
          <dd>{task.package.digest}</dd>
          {task.receipt && (
            <>
              <dt>设备提交摘要</dt>
              <dd>{task.receipt.digest}</dd>
            </>
          )}
        </dl>
      </details>
    </section>
  );
}
