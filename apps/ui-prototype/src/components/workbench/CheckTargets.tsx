import { CheckCircleIcon, WarningCircleIcon } from "@phosphor-icons/react";
import type { ProjectCheck } from "../../check-types";
export function CheckTargets({
  check,
  current,
  errors,
  warnings,
}: {
  check: ProjectCheck | null;
  current: boolean;
  errors: number;
  warnings: number;
}) {
  const ready = current && check?.report.desktopReady;
  return (
    <div className="wb-check-targets" aria-live="polite">
      <div className={ready ? "ready" : ""}>
        <strong>
          {ready ? <CheckCircleIcon /> : <WarningCircleIcon />}灯光编译
        </strong>
        <b>
          {!check
            ? "尚未检查"
            : !current
              ? "报告已过期"
              : ready
                ? "编译通过"
                : "需要修复"}
        </b>
        <span>
          {!check
            ? "按当前工程生成检查结果"
            : !current
              ? "工程或草稿已变化，请重新检查"
              : `${errors} 项错误 · ${warnings} 项提醒`}
        </span>
      </div>
      <div>
        <strong>音乐资源</strong>
        <b>
          {!check
            ? "尚未检查"
            : !current
              ? "报告已过期"
              : !check.audioResource
                ? "未使用音乐"
                : check.audioResource.resources.local.state !== "valid"
                  ? "需要恢复"
                  : check.audioResource.resources.companion.state !== "valid"
                    ? "随附文件待补齐"
                    : "文件完整"}
        </b>
        <span>分别核对本机音乐与工程随附文件</span>
      </div>
      <div>
        <strong>设备发布</strong>
        <b>独立核验</b>
        <span>生成播放包后，在节目安装中核验设备权限与安装结果</span>
      </div>
    </div>
  );
}
