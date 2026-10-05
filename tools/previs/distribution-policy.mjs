import { compareMacVersions } from "./mac-load-commands.mjs";

export const distributionRoles = ["desktop", "execution-host", "node", "game"];

export function assessDistributionPrerequisites(evidence) {
  if (!Array.isArray(evidence?.images) || evidence.images.length !== 4)
    throw new Error("发行前检查必须包含四个程序角色");
  if (
    new Set(evidence.images.map((image) => image.role)).size !== 4 ||
    distributionRoles.some(
      (role) => !evidence.images.some((image) => image.role === role),
    )
  )
    throw new Error("发行前程序角色缺失或重复");
  compareMacVersions(evidence.minimumMacOS, evidence.closureMinimumMacOS);
  const issues = [];
  const add = (image, code, message) =>
    issues.push({ role: image.role, program: image.program, code, message });
  for (const image of evidence.images) {
    for (const check of [image.strict, image.developerId, image.entitlements])
      if (
        !Number.isSafeInteger(check?.status) ||
        check.status < 0 ||
        check.status > 255
      )
        throw new Error("签名／工具状态元数据无效");
    if (
      typeof image.program !== "string" ||
      !image.program ||
      !Array.isArray(image.architectures)
    )
      throw new Error("程序路径或架构元数据无效");
    if (!image.architectures.includes("arm64") || image.executable !== true)
      add(image, "architecture", "程序不是可执行ARM64镜像");
    if (image.strict.status !== 0)
      add(image, "signature", "程序严格签名未通过");
    if (image.developerId.status !== 0)
      add(
        image,
        "developer-id",
        "实际Apple锚与Developer ID Application证书要求未通过",
      );
    const signed = image.signature;
    if (!signed) add(image, "signature-metadata", "未取得签名元数据");
    else {
      if (
        !Number.isSafeInteger(signed.flags) ||
        signed.flags < 0 ||
        signed.flags > 0xffffffff ||
        typeof signed.identifier !== "string"
      )
        throw new Error("签名标志或标识元数据无效");
      if ((signed.flags & 2) !== 0)
        add(image, "ad-hoc", "ad-hoc签名只可作内部资格，不能作为此发行候选");
      if (!/^[A-Z0-9]{10}$/.test(signed.teamId ?? ""))
        add(image, "team", "缺少有效签名Team标识");
      if ((signed.flags & 0x10000) === 0)
        add(image, "runtime", "缺少Hardened Runtime");
      if (
        typeof signed.timestamp !== "string" ||
        !signed.timestamp.trim() ||
        /^(none|not set)$/i.test(signed.timestamp)
      )
        add(image, "timestamp", "缺少安全时间戳，普通签署时间不替代");
      if (
        /^cn\.stagemaster\.acceptance\./.test(signed.identifier) ||
        /^com\.YourCompany\./.test(signed.identifier)
      )
        add(image, "internal-id", "仍使用内部验收或占位身份");
    }
    const rights = image.entitlements.values;
    if (
      image.entitlements.status !== 0 ||
      !rights ||
      typeof rights !== "object" ||
      Array.isArray(rights)
    )
      add(image, "entitlements", "无法确认实际程序资格，不按空资格通过");
    else {
      if (
        "com.apple.security.get-task-allow" in rights &&
        rights["com.apple.security.get-task-allow"] !== false
      )
        add(image, "debug", "调试资格开启或值畸形，不能用于此发行候选");
      if (
        Object.keys(rights).some((key) =>
          key.startsWith(
            "com.apple.security.temporary-exception.files.absolute-path.",
          ),
        )
      )
        add(
          image,
          "absolute-path",
          "固定绝对路径临时资格不是客户可移动权限方案",
        );
    }
    if (compareMacVersions(evidence.minimumMacOS, image.minimumMacOS) < 0)
      add(
        image,
        "minimum-os",
        `桌面最低系统${evidence.minimumMacOS}低于此程序${image.minimumMacOS}`,
      );
  }
  if (
    compareMacVersions(evidence.minimumMacOS, evidence.closureMinimumMacOS) < 0
  )
    add(
      evidence.images.find((image) => image.role === "game"),
      "minimum-os",
      `桌面最低系统${evidence.minimumMacOS}低于Game静态依赖${evidence.closureMinimumMacOS}`,
    );
  return {
    status: issues.length ? "blocked" : "static-prerequisites-passed",
    issues,
    releaseQualified: false,
    scope: "four-executables-and-game-static-closure-only",
    remaining: [
      "正式Shipping与最终权限方案",
      "完整嵌套代码与运行期加载",
      "公证／Gatekeeper",
      "许可及真实客户移位／GPU与代表任务",
    ],
  };
}
