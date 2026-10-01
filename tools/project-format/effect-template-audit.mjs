import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
// Supported v1 contains only objects, arrays, strings, booleans and safe integers.
function ordered(value) {
  if (Array.isArray(value)) return value.map(ordered);
  if (value && typeof value === 'object')
    return Object.fromEntries(Object.keys(value).sort().map(key=>[key,ordered(value[key])]));
  return value;
}
export function templateDigest(template) {
  return createHash('sha256').update(JSON.stringify(ordered(template))).digest('hex');
}
export function auditEffectTemplateSource(source, declared) {
  if (!source) return;
  assert(declared.has('lighting.effects.template-source@1'), '缺少灯效模板来源能力声明');
  assert(source.template.definition.name.trim(), '灯效模板名称不能为空');
  assert(source.sha256 === templateDigest(source.template), '灯效模板来源内容与摘要不一致');
}

export function auditTemplateIdentities(project) {
  const identities = new Map();
  for (const scene of project.lighting?.scenes ?? []) {
    for (const effect of scene.effects ?? []) {
      const source = effect.templateSource;
      if (!source) continue;
      const key = JSON.stringify([source.template.templateId, source.template.revision]);
      assert(!identities.has(key) || identities.get(key) === source.sha256,
        '相同灯效模板身份与修订对应了不同内容');
      identities.set(key, source.sha256);
    }
  }
}
