import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function document() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  const profile = p.lighting.profiles[0];
  profile.emitters = [{key:'wash',name:'染色光源'}];
  profile.attributes = ['red','green','blue','white'].map(key => ({key:`emitter.wash.${key}`,mix:'ltp',valueType:{kind:'normalized'},default:{kind:'normalized',value:65535}}));
  profile.channels = profile.attributes.map((a,i) => ({attribute:a.key,encoding:'u8',offsets:[i]}));
  profile.footprint = 4;
  p.lighting.scenes = []; p.lighting.presets = []; p.lighting.sequences = [];
  p.entryPoints = [];
  p.requires.push({key:'lighting.fixture-emitters',version:1});
  return p;
}
test('独立 RGBW 归属通过格式检查', () => assert.doesNotThrow(() => auditProject(document())));
for (const [name, mutate, pattern] of [
  ['缺少能力',p => p.requires.pop(),/缺少能力/],
  ['未知版本',p => p.requires.at(-1).version = 2,/缺少能力/],
  ['缺少单元',p => delete p.lighting.profiles[0].emitters,/单元定义/],
  ['未知归属',p => p.lighting.profiles[0].emitters[0].key = 'other',/归属/],
  ['重复标识',p => p.lighting.profiles[0].emitters.push({...p.lighting.profiles[0].emitters[0]}),/重复/],
  ['空名称',p => p.lighting.profiles[0].emitters[0].name = ' ',/名称/],
  ['不完整 RGB',p => p.lighting.profiles[0].attributes.pop() && p.lighting.profiles[0].attributes.pop(),/完整/],
  ['声控伪装',p => p.lighting.profiles[0].attributes[0].key = 'emitter.wash.fixture-program',/完整/],
  ['错误混合',p => p.lighting.profiles[0].attributes[0].mix = 'htp',/混合/],
]) test(`独立光源拒绝${name}`, () => { const p = document(); mutate(p); assert.throws(() => auditProject(p),pattern); });
