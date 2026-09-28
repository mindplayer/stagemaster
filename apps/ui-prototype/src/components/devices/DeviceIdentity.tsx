import type { DeviceDescription } from "../../device-types";

/** Read-only host metadata. Declared support never enables a privileged action. */
export function DeviceIdentity({ description }: { description: DeviceDescription | null }) {
  if (!description) return <p>设备未提供身份和能力描述，当前仅可进行连接诊断。</p>;
  return <section aria-label="设备身份与能力">
    <dl>
      <div><dt>设备型号</dt><dd>{description.modelName}</dd></div>
      <div><dt>固件版本</dt><dd>{description.firmware}</dd></div>
      <div><dt>固件声明</dt><dd>{description.declaredFunctions.join("、")}</dd></div>
    </dl>
    {description.unknownCapabilities !== 0 && <p>设备包含本软件尚未识别的能力，请核对软件与固件版本。</p>}
    <details className="wb-device-details">
      <summary>设备标识与容量</summary>
      <dl>
        <div><dt>设备标识</dt><dd>{description.deviceId}</dd></div>
        <div><dt>本次启动</dt><dd>{description.bootId}</dd></div>
        {description.limits.packageBytes > 0 && <>
          <div><dt>节目包上限</dt><dd>{description.limits.packageBytes.toLocaleString()} 字节</dd></div>
          <div><dt>节目数量上限</dt><dd>{description.limits.programs}</dd></div>
        </>}
        {description.limits.universes > 0 && <div><dt>线路上限</dt><dd>{description.limits.universes}</dd></div>}
      </dl>
      <p>设备标识用于区分设备；能力声明不代表已完成认证或取得节目安装权限。</p>
    </details>
  </section>;
}
