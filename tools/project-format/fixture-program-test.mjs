import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';

function document() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  const profile = p.lighting.profiles[0];
  profile.attributes.push({key:'fixture-program',mix:'ltp',valueType:{kind:'function'},default:{kind:'function',functionKey:'external',position:0}});
  profile.channels.push({attribute:'fixture-program',encoding:'u8',offsets:[profile.footprint],functions:[
    {key:'external',name:'外部通道控制',mode:'slot',dmxFrom:0,dmxTo:59,dmxDefault:0},
    {key:'auto.3',name:'自动 3',mode:'slot',dmxFrom:60,dmxTo:84,dmxDefault:60},
    {key:'sound.3',name:'声控 3',mode:'slot',dmxFrom:160,dmxTo:184,dmxDefault:160},
  ]});
  profile.footprint++;
  p.lighting.patches.forEach((patch, i) => patch.address = 1+i*16);
  p.requires.push({key:'lighting.fixture-functions',version:1},{key:'lighting.fixture-programs',version:1});
  const fixture = p.lighting.fixtures.find(f => f.profileId === profile.id);
  p.lighting.scenes[0].assignments.push({target:{fixtureId:fixture.id,attribute:'fixture-program'},operation:'set',source:{kind:'literal',value:{kind:'function',functionKey:'external',position:0}}});
  return p;
}
test('禁用区间仅作资料，外部控制以离散语义通过格式检查', () => assert.doesNotThrow(() => auditProject(document())));
for (const [name, mutate, pattern] of [
  ['能力缺失',p=>p.requires.pop(),/内置程序缺少能力/],
  ['未知版本',p=>p.requires.at(-1).version=2,/内置程序缺少能力/],
  ['连续百分比',p=>p.lighting.profiles[0].attributes.at(-1).valueType.kind='normalized',/普通百分比/],
  ['自动默认',p=>p.lighting.profiles[0].attributes.at(-1).default.functionKey='auto.3',/默认须为外部/],
  ['缺少外部档位',p=>p.lighting.profiles[0].channels.at(-1).functions.shift(),/外部通道/],
  ['动态区间',p=>p.lighting.profiles[0].channels.at(-1).functions[1].mode='range',/固定档位/],
  ['混入复位',p=>p.lighting.profiles[0].channels.at(-1).functions[1].key='reset',/不能承载复位/],
  ['场景非零位置',p=>p.lighting.scenes[0].assignments.at(-1).source.value.position=1,/必须为零/],
  ['声控场景',p=>p.lighting.scenes[0].assignments.at(-1).source.value.functionKey='sound.3',/已屏蔽/],
  ['自走场景',p=>p.lighting.scenes[0].assignments.at(-1).source.value.functionKey='auto.3',/已屏蔽/],
]) test(`内置程序拒绝${name}`,()=>{const p=document(); mutate(p); assert.throws(()=>auditProject(p),pattern);});
