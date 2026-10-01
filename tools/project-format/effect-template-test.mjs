import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
import { templateDigest } from './effect-template-audit.mjs';
function document() {
  const p=structuredClone(loadExamples().find(p=>p.project?.name==='单路灯光示例'));
  const template={format:'stagemaster-effect-template',formatVersion:1,
    templateId:'19999999-0000-4000-8000-000000000001',revision:'19999999-0000-4000-8000-000000000002',
    definition:{name:'亮度呼吸',recipe:{kind:'intensity-wave',waveform:'smooth',low:0,high:65535,dutyPercent:50},
      timing:{periodMs:2000,phaseDegrees:0,spreadDegrees:360,reverseOrder:false}}};
  p.requires.push({key:'lighting.effects.basic',version:1},{key:'lighting.effects.template-source',version:1});
  p.lighting.scenes[0].effects=[{id:'19999999-0000-4000-8000-000000000003',name:'本地名称',enabled:true,
    fixtureIds:[p.lighting.fixtures[0].id],waveform:'smooth',periodMs:3000,phaseDegrees:0,spreadDegrees:360,reverse:false,dutyPercent:50,
    channels:[{attribute:'dimmer',low:0,high:65535}],templateSource:{template,sha256:templateDigest(template)}}];
  return p;
}
const source=p=>p.lighting.scenes[0].effects[0].templateSource;
test('来源模板与本地参数分别保存，重复来源不构成重复工程身份',()=>{
  const p=document(),e=structuredClone(p.lighting.scenes[0].effects[0]);
  e.id='19999999-0000-4000-8000-000000000004';e.enabled=false;
  p.lighting.scenes[0].effects.push(e);assert.doesNotThrow(()=>auditProject(p));
});
for(const [name,mutate,pattern] of [
  ['篡改摘要',p=>source(p).sha256='0'.repeat(64),/摘要/],
  ['缺少能力',p=>p.requires.pop(),/能力声明/],
  ['未知类型',p=>source(p).template.definition.recipe.kind='laser-program',/Schema/],
  ['夹带通道',p=>source(p).template.definition.recipe.channel=1,/Schema/],
  ['空白名称',p=>source(p).template.definition.name=' ',/名称/],
  ['幅度越界',p=>source(p).template.definition.recipe.high=65536,/Schema/],
]) test(`灯效模板拒绝${name}`,()=>{const p=document();mutate(p);assert.throws(()=>auditProject(p),pattern);});

test('相同模板修订内容冲突，必须建立新修订',()=>{
  const p=document(),e=structuredClone(p.lighting.scenes[0].effects[0]);
  e.id='19999999-0000-4000-8000-000000000004';e.enabled=false;
  e.templateSource.template.definition.timing.periodMs=3000;
  e.templateSource.sha256=templateDigest(e.templateSource.template);
  p.lighting.scenes[0].effects.push(e);
  assert.throws(()=>auditProject(p),/不同内容/);
  e.templateSource.template.revision='19999999-0000-4000-8000-000000000005';
  e.templateSource.sha256=templateDigest(e.templateSource.template);
  assert.doesNotThrow(()=>auditProject(p));
});
