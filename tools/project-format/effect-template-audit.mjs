import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
// Supported recipes contain only objects, arrays, strings, booleans and safe integers.
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
  if (source.template.formatVersion === 2) {
    assert(declared.has('lighting.effects.template-keyframes@1'), '缺少关键帧灯效模板来源能力声明');
    const frames = source.template.definition.recipe.keyframes;
    assert(frames[0].position === 0 && frames.every((f,i)=>i===0 || f.position>frames[i-1].position),
      '关键帧须从 0% 开始并按时间递增');
  }
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
