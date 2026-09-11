/** 仅供类型检查的调用样例；无实现、不自动连接设备、不执行真实输出。 */
import {
  connectStage, counter, decimal, LocalIpcTransport, RemoteTransport,
  type ActiveRun, type BuildArtifact, type CloudClient, type ControlTicket,
  type Id, type IdFactory, type JobRef, type JobsApi, type Meta,
  type ProfileRef, type ProjectRef, type Result, type StageClient,
} from "./contracts";

function must<T>(result: Result<T>): T {
  if (!result.ok) throw result.error;
  return result.value;
}
function meta(ids: IdFactory): Meta {
  return { protocol: "draft-0.3", commandId: ids.next<"command">() };
}
function reportCleanup<T>(result: Result<T>): void {
  // 示例用稳定错误码提示；产品应独立展示清理失败，不覆盖原操作的错误。
  if (!result.ok) console.warn("资源清理未确认，需对账或等待服务端到期回收", result.error.code);
}
/** 演示用有界等待；超时只停止观察，不能据此重发原动作。 */
export async function awaitJob<T>(jobs: JobsApi, job: JobRef<T>): Promise<T> {
  for (let poll = 0; poll < 30; poll += 1) {
    const state = must(await jobs.wait(job, { timeoutMs: 1000 }));
    switch (state.state) {
      case "succeeded": return state.value;
      case "failed": throw state.error;
      case "cancelled": throw new Error("任务已取消");
      case "queued": case "running": break;
    }
  }
  throw new Error(`等待超时，按 jobId 对账，不重复提交：${job.jobId}`);
}
export async function awaitApplied(client: StageClient, ticket: ControlTicket): Promise<void> {
  for (let poll = 0; poll < 20; poll += 1) {
    const outcome = must(await client.control.outcome(ticket));
    switch (outcome.state) {
      case "applied": return;
      case "rejected": throw outcome.error;
      case "unknown": throw new Error("执行结果未知，先读取状态，禁止自动重发 Go");
      case "accepted": case "scheduled":
        await new Promise<void>(resolve => setTimeout(resolve, 100));
        break;
    }
  }
  throw new Error("尚未确认已应用；保留 ticket 查询结果");
}

// 1. 相同业务函数通过不同 transport 连接；凭据函数只在客户端本地调用。
export async function connectLocal(endpoint: string): Promise<StageClient> {
  return must(await connectStage(new LocalIpcTransport({ endpoint })));
}
export async function connectRemote(url: string, credentialProvider: () => Promise<string>): Promise<StageClient> {
  return must(await connectStage(new RemoteTransport({ url, credentialProvider })));
}

// 2. 创建 → 配适 → Programmer → 记录 → 保存 → 编译 → 准备 → 激活 → Go。
// profile、element、属性键、target、端点来自已验证的资源/设备查询，并非硬编码硬件地址。
export async function authorAndPlay(client: StageClient, ids: IdFactory, input: {
  readonly profile: ProfileRef;
  readonly elementId: Id<"element">;
  readonly connectionId: Id<"fixture-connection">;
  readonly intensityAttribute: string;
  readonly targetId: Id<"target">;
  readonly domainId: Id<"execution-domain">;
  readonly dmxEndpointId: Id<"endpoint">;
}): Promise<{ readonly run: ActiveRun; readonly project: ProjectRef; readonly build: BuildArtifact }> {
  let project = must(await client.projects.create({ ...meta(ids), name: "接口联调节目" }));
  const fixtureId = ids.next<"fixture">();
  const groupId = ids.next<"group">();
  const presetId = ids.next<"preset">();
  const sequenceId = ids.next<"sequence">();
  const playbackId = ids.next<"playback">();
  const cueId = ids.next<"cue">();
  project = must(await client.projects.edit({
    ...meta(ids), base: project,
    operations: [
      { kind: "add-fixture", fixtureId, name: "面光 1", profile: input.profile },
      { kind: "create-group", groupId, name: "面光", fixtures: [fixtureId] },
      { kind: "create-preset", presetId, name: "面光 65%", values: [{
        address: { fixtureId, elementId: input.elementId, attribute: input.intensityAttribute },
        source: { kind: "literal", value: { kind: "normalized", value: decimal("0.65") } },
      }] },
      { kind: "create-sequence", sequenceId, name: "开场" },
      { kind: "configure-playback", playbackId, sequenceId },
    ],
  })).project;

  let session = must(await client.sessions.openBlind({ ...meta(ids), project }));
  try {
    const sessionEdit = must(await client.sessions.edit({ ...meta(ids), base: session, operations: [
      { kind: "select-group", groupId },
      { kind: "apply-preset", presetId },
    ] }));
    session = sessionEdit.session;
    if (sessionEdit.liveApplication.kind !== "blind") throw new Error("预期为盲编上下文");
    const programmer = must(await client.sessions.snapshot(session));
    project = must(await client.projects.edit({ ...meta(ids), base: project, operations: [
      { kind: "record-cue", sequenceId, cueId, number: "1", name: "开场面光",
        programmer, values: "retain-references" },
    ] })).project;
  } finally {
    reportCleanup(await client.sessions.close({ ...meta(ids), sessionId: session.sessionId }));
  }

  let binding = must(await client.bindings.create({ ...meta(ids), projectId: project.projectId, targetId: input.targetId }));
  binding = must(await client.bindings.edit({ ...meta(ids), base: binding, project, routes: [{
    kind: "dmx", fixtureId, connectionId: input.connectionId, endpointId: input.dmxEndpointId, universe: 1, startAddress: 1,
  }] }));
  const saved = await awaitJob(client.jobs, must(await client.projects.save({ ...meta(ids), project })));
  if (saved.saved.revisionId !== project.revisionId) throw new Error("保存确认不是请求的修订");
  const target = must(await client.targets.inspect({ targetId: input.targetId, domainId: input.domainId }));
  const expectedRun = target.run;
  const build = await awaitJob(client.jobs, must(await client.builds.compile({ ...meta(ids), project, binding, target, domainId: input.domainId })));
  const prepared = await awaitJob(client.jobs, must(await client.deployment.prepare({ ...meta(ids), artifact: build, expectedRun })));
  // 若下面请求超时，用 job/commandId 查询；不把超时解释为激活失败后盲目重试。
  const activation = await awaitJob(client.jobs, must(await client.deployment.activate({
    ...meta(ids), preparedId: prepared.preparedId, expectedRun, when: { kind: "next-boundary" },
  })));
  // 这里只激活一个执行域；跨节点发布需要逐域跟踪，不能复用为全场原子提交。
  const run = activation.run;
  const lease = must(await client.control.acquire({
    ...meta(ids), run, scope: { kind: "playback", playbackId }, takeover: "deny-if-owned",
  }));
  try {
    const ticket = must(await client.control.submit({
      ...meta(ids), lease, sequence: counter("1"), action: { kind: "go", playbackId },
    }));
    await awaitApplied(client, ticket);
  } finally {
    // 归还“操作该执行器”的租约；不等同于 Release Playback，也不撤销物理输出所有权。
    reportCleanup(await client.control.relinquish({ ...meta(ids), lease }));
  }
  return { run, project, build };
}

// 3. 手机端先请求修改影响预览；提交绑定基准修订，不直接改对象字段。
export async function renameFromRemote(client: StageClient, ids: IdFactory, project: ProjectRef, objectId: string): Promise<ProjectRef> {
  const preview = must(await client.projects.previewEdit({
    ...meta(ids), base: project, operations: [{ kind: "rename-object", objectId, name: "新版名称" }],
  }));
  if (preview.problems.length > 0) throw new Error("变更预览有阻断项");
  return must(await client.projects.commitPreview({
    ...meta(ids), preview: preview.token, expected: preview.base,
  })).project;
}

// 4. 仅定位一个同步组；运输代次由权威状态查询取得，其他组不重置。
export async function seekShowGroup(client: StageClient, ids: IdFactory, run: ActiveRun, syncGroupId: Id<"sync-group">): Promise<void> {
  const snapshot = must(await client.observe.snapshot(run));
  const transport = snapshot.transports.find(item => item.syncGroupId === syncGroupId);
  if (!transport) throw new Error("同步组不存在");
  const lease = must(await client.control.acquire({
    ...meta(ids), run, scope: { kind: "sync-group", syncGroupId }, takeover: "deny-if-owned",
  }));
  try {
    const ticket = must(await client.control.submit({
      ...meta(ids), lease, sequence: counter("1"),
      action: { kind: "seek", transport, position: { ticks: "12000", ticksPerSecond: counter("1000") } },
    }));
    await awaitApplied(client, ticket); // 仅本地组状态；外部视频定位另查 external.outcome/state，不能等同完成。
  } finally {
    reportCleanup(await client.control.relinquish({ ...meta(ids), lease }));
  }
}

// 5. 订阅是客户端本地迭代器；网络上传游标和 DTO，不传函数／迭代器实例。
export async function watchUntilGap(client: StageClient, run: ActiveRun): Promise<void> {
  const snapshot = must(await client.observe.snapshot(run));
  const subscription = must(await client.observe.subscribe({ run, after: snapshot.cursor }));
  let cursor = snapshot.cursor;
  try {
    for await (const event of subscription.events) {
      if (event.kind === "resync-required") break; // 返回调用方重新取 snapshot 并建立订阅。
      if (event.kind === "snapshot") cursor = event.snapshot.cursor;
      if (event.kind === "delta") {
        if (event.base !== cursor) break;
        cursor = event.next;
      }
    }
  } finally {
    await subscription.close();
  }
}

// 6. 云端“分配目标包”不等于现场激活。
export async function publishForDevice(client: StageClient, cloud: CloudClient, ids: IdFactory, savedProject: ProjectRef, build: BuildArtifact, deviceId: Id<"device">): Promise<void> {
  const exported = await awaitJob(client.jobs, must(await client.exports.deployment({ ...meta(ids), savedProject, build })));
  const upload = must(await cloud.openUpload({ ...meta(ids), manifest: exported.manifest }));
  await awaitJob(client.jobs, must(await client.transfers.upload({ ...meta(ids), source: exported.resource, upload })));
  const verified = await awaitJob(cloud.jobs, must(await cloud.verifyUpload({ ...meta(ids), uploadId: upload.uploadId })));
  if (verified.manifestHash !== exported.manifest.manifestHash) throw new Error("上传校验清单不一致");
  const published = await awaitJob(cloud.jobs, must(await cloud.publish({ ...meta(ids), verified })));
  if (published.kind !== "deployment") throw new Error("只有执行包可以分配给现场设备");
  const assignment = must(await cloud.assign({ ...meta(ids), deployment: published, deviceId }));
  if (assignment.state !== "desired") throw new Error("分发状态异常");
  const state = must(await cloud.inspectDevice(deviceId));
  // UI 分别显示 state.desired / downloaded / prepared / active 和最后上报时间。
  // 不因为 assign 成功而更新 UI 的 active，也不自动调用本地 activate。
  void state;
}
