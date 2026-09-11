/** 编译期反例：只验证契约隔离，不能证明服务端授权、时序或业务语义。 */
import type { ActiveRun, CloudClient, ControlApi, ControlLease, Id, Meta, PreviewRef,
  ProjectApi, ProjectRef, Route, RunCursor, StageClient } from "./contracts";
import type { ExternalAction, ExternalApi, ExternalControlContext, MonitorApi,
  MonitorBindingSpec, MonitorSession, MonitorSourceRef } from "./external-contracts";
import type { SurfaceCoordinator, SurfaceDriver, SurfaceInput, SurfaceRef } from "./surface-contracts";

export function identityBoundaries(fixture: Id<"fixture">, cue: Id<"cue">): void {
  // @ts-expect-error 灯具身份不能作为 Cue 身份。
  const invalid: Id<"cue"> = fixture;
  void invalid;
  void cue;
}
export function previewHasNoOutputAuthority(preview: PreviewRef): void {
  // @ts-expect-error 预演句柄不是现场运行实例。
  const invalid: ActiveRun = preview;
  void invalid;
}
export function emptyPlanCannotBeActive(cursor: RunCursor): void {
  // @ts-expect-error 未激活执行域不能冒充 ActiveRun。
  const invalid: ActiveRun = cursor;
  void invalid;
}
export function generationIsNotANumber(run: ActiveRun): void {
  // @ts-expect-error 代次使用字符串计数器，禁止普通 JS Number。
  const invalid: ActiveRun = { ...run, planGeneration: 9007199254740993 };
  void invalid;
}
export function projectEditNeedsRevision(api: ProjectApi, meta: Meta): void {
  // @ts-expect-error 缺少基准工程修订。
  void api.edit({ ...meta, operations: [] });
}
export function controlNeedsLease(api: ControlApi, meta: Meta, playbackId: Id<"playback">): void {
  // @ts-expect-error 缺少控制租约与顺序号。
  void api.submit({ ...meta, action: { kind: "go", playbackId } });
}
export function objectsAreReadonly(project: ProjectRef, lease: ControlLease): void {
  // @ts-expect-error 客户端不能原地改服务返回的修订引用。
  project.revisionId = "replacement";
  // @ts-expect-error 控制租约不包含物理设备句柄。
  void lease.deviceHandle;
}

export function monitoringDoesNotGrantControl(session: MonitorSession, external: ExternalApi, meta: Meta): void {
  // @ts-expect-error 监看令牌不能用来续租外部控制权。
  void external.renew({ ...meta, lease: session });
}
export function noProgramMediaOutput(api: MonitorApi, projectApi: ProjectApi, project: ProjectRef, meta: Meta): void {
  // @ts-expect-error 监看服务不提供主扩音量控制。
  void api.setMainGain(0);
  // @ts-expect-error 旧版内置音频总线操作已不在当前契约中。
  void projectApi.edit({ ...meta, base: project, operations: [{ kind: "create-audio-bus" }] });
}
export function noRawProtocolEscape(): void {
  // @ts-expect-error 业务动作不能直接发送任意 OSC 字符串。
  const action: ExternalAction = { kind: "send-osc", address: "/anything" };
  void action;
}

export function surfaceIsNotControlAuthority(surface: SurfaceRef, api: ControlApi, meta: Meta): void {
  // @ts-expect-error 控制面连接身份不能作为业务控制租约。
  void api.renew({ ...meta, lease: surface });
}
export function rawHardwareIsNotPublicRpc(client: StageClient, input: SurfaceInput): void {
  // @ts-expect-error 普通客户端不能注入原始硬件输入。
  void client.surfaces.ingest(input);
}

export function panelConnectionDoesNotRequireRun(coordinator: SurfaceCoordinator, driver: SurfaceDriver,
  profile: Id<"surface-profile">, run: ActiveRun): void {
  // @ts-expect-error 连接面板需宿主操作会话，不能把运行实例当身份。
  void coordinator.attach(driver, profile, run);
}
export function monitorSessionIsNotPersistentSource(surfaceId: Id<"video-surface">, source: MonitorSourceRef): void {
  // @ts-expect-error 持久监看配置不能保存带重连代次的临时源引用。
  const invalid: MonitorBindingSpec = { surfaceId, source };
  void invalid;
  // @ts-expect-error 监看布局已移出影响节目执行的路由。
  const route: Route = { kind: "monitor-surface", surfaceId, source };
  void route;
}
export function projectPackageIsNotExecutable(cloud: CloudClient, meta: Meta,
  packageId: Id<"package">, deviceId: Id<"device">): void {
  const project = { kind: "project" as const, packageId };
  // @ts-expect-error 工程交换包不是设备执行包。
  void cloud.assign({ ...meta, deployment: project, deviceId });
}
export function showActionsNeedTransportGeneration(run: ActiveRun): void {
  // @ts-expect-error 仅有节目计划不足以判断 Seek 后外部动作是否过期。
  const context: ExternalControlContext = { kind: "show", transport: { run } };
  void context;
}
