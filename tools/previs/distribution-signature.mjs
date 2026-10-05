// Text describes signatures; only the separate native requirement verifies trust.
export function parseDistributionSignature(text) {
  if (
    typeof text !== "string" ||
    text.length > 4 * 1024 * 1024 ||
    text.includes("\0")
  )
    throw new Error("签名元数据无效或超限");
  const one = (pattern, required = true) => {
    const matches = [...text.matchAll(pattern)];
    if (matches.length > 1 || (required && matches.length !== 1))
      throw new Error("签名元数据缺失或重复");
    const value = matches[0]?.[1];
    if (value !== undefined && (!value || /[\x00-\x1f\x7f]/.test(value)))
      throw new Error("签名元数据字段无效");
    return value ?? null;
  };
  const identifier = one(/^Identifier=(.*)$/gm);
  const directory = one(/^CodeDirectory (.*)$/gm);
  const flagFields = [
    ...directory.matchAll(/\bflags=0x([0-9a-fA-F]{1,8})(?=\(|\s|$)/g),
  ];
  if (flagFields.length !== 1) throw new Error("签名标志缺失或无效");
  const flags = flagFields[0][1];
  const team = one(/^TeamIdentifier=(.*)$/gm);
  const timestamp = one(/^Timestamp=(.*)$/gm, false);
  if (team !== "not set" && !/^[A-Z0-9]{10}$/.test(team))
    throw new Error("签名Team元数据无效");
  return {
    identifier,
    flags: Number.parseInt(flags, 16),
    adHoc: (Number.parseInt(flags, 16) & 2) !== 0,
    hardenedRuntime: (Number.parseInt(flags, 16) & 0x10000) !== 0,
    teamId: team === "not set" ? null : team,
    timestamp,
    authorities: [...text.matchAll(/^Authority=(.+)$/gm)].map(
      (match) => match[1],
    ),
  };
}

// Apple anchor + intermediate Developer ID + Application leaf; not a name match.
export const developerIdRequirement =
  "=anchor apple generic and certificate 1[field.1.2.840.113635.100.6.2.6] exists and certificate leaf[field.1.2.840.113635.100.6.1.13] exists";
