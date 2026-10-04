// Authoring contract only; program execution and DMX encoding belong to Rust.
export function auditFixtureProgram(project, attribute, channel) {
  const assert = (ok, message) => { if (!ok) throw new Error(message); };
  assert(project.requires.some(c => c.key === 'lighting.fixture-programs' && c.version === 1), '内置程序缺少能力声明');
  assert(attribute.valueType.kind === 'function', '内置程序不能使用普通百分比');
  assert(channel?.functions?.some(f => f.key === 'external'), '内置程序须定义外部通道控制档位');
  assert(channel.functions.every(f => f.mode === 'slot' && (f.key === 'external' || /^(auto|sound)\./.test(f.key))), '内置程序只支持外部控制、自动或声控固定档位，不能承载复位');
  assert(attribute.default.kind === 'function' && attribute.default.functionKey === 'external' && attribute.default.position === 0, '内置程序默认须为外部通道控制');
}
