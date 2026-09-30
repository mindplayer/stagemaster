import type { AudioResourceCheck, ResourceFileHealth } from "../../check-types";
function stateLabel(health: ResourceFileHealth, companion = false) {
  switch (health.state) {
    case "valid":
      return companion ? "随附文件完整" : "本机文件完整";
    case "missing":
      return companion ? "缺少随附文件" : "找不到音乐";
    case "invalid":
      return "文件校验失败";
    case "notSaved":
      return "尚未保存工程";
  }
}
export function CheckAudioResources({
  resource,
  current,
  disabled,
  onLocate,
  onSave,
}: {
  resource: AudioResourceCheck | null;
  current: boolean;
  disabled: boolean;
  onLocate(): void;
  onSave(): void;
}) {
  if (!resource)
    return (
      <p className="wb-check-caption">
        {current
          ? "当前工程未使用音乐。"
          : "历史检查未使用音乐，请重新检查当前工程。"}
      </p>
    );
  const { local, companion, localSource } = resource.resources;
  return (
    <section className="check-audio-resources" aria-label="音乐资源检查">
      <header>
        <div>
          <h3>音乐资源</h3>
          <p>{resource.fileName}</p>
        </div>
        <div className="wb-check-actions">
          {local.state === "valid" && companion.state !== "valid" && (
            <button disabled={disabled || !current} onClick={onSave}>
              保存并补齐
            </button>
          )}
          <button disabled={disabled || !current} onClick={onLocate}>
            前往音乐
          </button>
        </div>
      </header>
      <div className="check-audio-copies">
        <article data-state={current ? local.state : "stale"}>
          <strong>本机使用</strong>
          <b>{current ? stateLabel(local) : "报告已过期"}</b>
          <p>
            {local.state === "invalid"
              ? local.message
              : local.state === "valid"
                ? `已核对音乐内容 · ${localSource === "cache" ? "本机缓存" : "工程随附文件"}`
                : "前往音乐，重新定位原文件。"}
          </p>
        </article>
        <article data-state={current ? companion.state : "stale"}>
          <strong>随工程携带</strong>
          <b>{current ? stateLabel(companion, true) : "报告已过期"}</b>
          <p>
            {companion.state === "valid" ? (
              "移动工程时一起携带同名 .assets 文件夹。"
            ) : (
              <>
                {companion.state === "invalid" && `${companion.message}。`}
                {local.state === "valid"
                  ? "保存工程可补齐同名 .assets 文件夹内的音乐。"
                  : "先恢复本机音乐，再保存工程补齐随附文件。"}
              </>
            )}
          </p>
        </article>
      </div>
      <p className="wb-check-caption">
        文件完整性为检查时结果，不含音频输出测试。
      </p>
    </section>
  );
}
