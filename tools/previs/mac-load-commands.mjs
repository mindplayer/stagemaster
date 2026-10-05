const dependencyCommands = new Set([
  "LC_LOAD_DYLIB",
  "LC_LOAD_WEAK_DYLIB",
  "LC_REEXPORT_DYLIB",
  "LC_LOAD_UPWARD_DYLIB",
  "LC_LAZY_LOAD_DYLIB",
]);

export function macVersion(value) {
  if (typeof value !== "string" || !/^\d+(?:\.\d+){0,2}$/.test(value))
    throw new Error("Mach-O 最低系统版本无效");
  const numbers = value.split(".").map(Number);
  if (numbers.some((number) => !Number.isSafeInteger(number) || number > 65535))
    throw new Error("Mach-O 最低系统版本无效");
  return [...numbers, ...Array(3 - numbers.length).fill(0)];
}

export function compareMacVersions(left, right) {
  const a = macVersion(left);
  const b = macVersion(right);
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return 0;
}

// Read load commands, not `otool -L`: a dylib's own install ID is not a dependency.
export function parseMacLoadCommands(text) {
  const blocks = text.split(/^Load command \d+\s*$/m).slice(1);
  const dependencies = [];
  const rpaths = [];
  const versions = [];
  const commands = [];
  const field = (block, key) => {
    const value = block.match(
      new RegExp(`^\\s*${key} (.+?)(?: \\(offset \\d+\\))?\\s*$`, "m"),
    )?.[1];
    if (!value || /[\x00-\x1f\x7f]/.test(value))
      throw new Error(`Mach-O ${key} 元数据缺失或无效`);
    return value;
  };
  for (const block of blocks) {
    const command = block.match(/^\s*cmd (\S+)\s*$/m)?.[1];
    if (!command) throw new Error("Mach-O 加载命令无效");
    commands.push(command);
    if (dependencyCommands.has(command))
      dependencies.push({
        name: field(block, "name"),
        weak: command === "LC_LOAD_WEAK_DYLIB",
        command,
      });
    if (command === "LC_RPATH") rpaths.push(field(block, "path"));
    if (command === "LC_BUILD_VERSION") {
      if (!["1", "MACOS", "macos"].includes(field(block, "platform")))
        throw new Error("Mach-O 不是 macOS 镜像");
      versions.push(field(block, "minos"));
    }
    if (command === "LC_VERSION_MIN_MACOSX")
      versions.push(field(block, "version"));
    if (/^LC_VERSION_MIN_(IPHONEOS|TVOS|WATCHOS)$/.test(command))
      throw new Error("Mach-O 不是 macOS 镜像");
    if (command === "LC_DYLD_ENVIRONMENT")
      throw new Error("Mach-O 含动态加载环境覆盖，不接受为独立组件");
  }
  if (versions.length !== 1) throw new Error("Mach-O 最低系统版本缺失或重复");
  macVersion(versions[0]);
  return {
    minimumMacOS: versions[0],
    dependencies,
    rpaths,
    executable:
      commands.includes("LC_MAIN") || commands.includes("LC_UNIXTHREAD"),
    dylib: commands.includes("LC_ID_DYLIB"),
  };
}
