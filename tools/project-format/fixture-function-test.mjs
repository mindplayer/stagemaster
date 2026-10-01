import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function document() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  const profile = p.lighting.profiles[0];
  profile.attributes.push({key:'shutter',mix:'ltp',valueType:{kind:'function'},default:{kind:'function',functionKey:'open',position:0}});
  profile.channels.push({attribute:'shutter',encoding:'u8',offsets:[profile.footprint],functions:[
    {key:'open',name:'常开',mode:'slot',dmxFrom:0,dmxTo:15,dmxDefault:10},
    {key:'strobe',name:'频闪',mode:'range',dmxFrom:32,dmxTo:255,dmxDefault:100} ]});
  profile.footprint++;
  p.lighting.patches.forEach((patch, i) => patch.address = 1 + i * 16);
  p.requires.push({key:'lighting.fixture-functions',version:1});
  const fixture = p.lighting.fixtures.find(f => f.profileId === profile.id);
  p.lighting.scenes[0].assignments.push({target:{fixtureId:fixture.id,attribute:'shutter'},operation:'set',source:{kind:'literal',value:{kind:'function',functionKey:'strobe',position:32768}}});
  return p;
}
test('功能区间工程结构及语义可离线审查，允许保留通道空隙', () => assert.doesNotThrow(() => auditProject(document())));
for (const [name, mutate, pattern] of [
  ['能力缺失', p => p.requires.pop(), /能力声明/],
  ['区间重叠', p => p.lighting.profiles[0].channels.at(-1).functions[1].dmxFrom = 15, /重叠/],
  ['代表值越界', p => p.lighting.profiles[0].channels.at(-1).functions[0].dmxDefault = 16, /代表值/],
  ['精度越界', p => p.lighting.profiles[0].channels.at(-1).functions[1].dmxTo = 256, /精度/],
  ['默认功能不存在', p => p.lighting.profiles[0].attributes.at(-1).default.functionKey = 'missing', /未知灯具功能/],
  ['固定档位非零位置', p => p.lighting.profiles[0].attributes.at(-1).default.position = 1, /必须为零/],
  ['场景引用空隙', p => p.lighting.scenes[0].assignments.at(-1).source.value.functionKey = 'missing', /未知灯具功能/],
  ['重复功能键', p => p.lighting.profiles[0].channels.at(-1).functions[1].key = 'open', /重复/],
  ['越界区间位置', p => p.lighting.scenes[0].assignments.at(-1).source.value.position = 65536, /Schema/],
]) test(`功能区间拒绝${name}`, () => { const p = document(); mutate(p); assert.throws(() => auditProject(p), pattern); });

test('色盘外观可选且严格限于固定档位，要求显式能力', () => {
  const p = document(), profile = p.lighting.profiles[0];
  const channel = profile.channels.at(-1), attribute = profile.attributes.at(-1);
  channel.attribute = attribute.key = 'color-wheel';
  p.lighting.scenes[0].assignments.at(-1).target.attribute = 'color-wheel';
  channel.functions[0].appearance = {kind:'color',colors:['#ff0000','#0000FF']};
  p.requires.push({key:'lighting.fixture-wheel-appearance',version:1});
  assert.doesNotThrow(() => auditProject(p));
  const noCapability = structuredClone(p); noCapability.requires.pop();
  assert.throws(() => auditProject(noCapability), /外观缺少能力/);
  for (const colors of [[], ['red'], ['#ffffff','#000000','#ff0000']]) {
    const invalid = structuredClone(p); invalid.lighting.profiles[0].channels.at(-1).functions[0].appearance.colors = colors;
    assert.throws(() => auditProject(invalid), /Schema/);
  }
  channel.functions[1].appearance = {kind:'open'};
  assert.throws(() => auditProject(p), /固定档位/);
});
