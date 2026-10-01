import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function document() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  p.requires.push({key:'lighting.position-reference',version:1});
  // Historical record retained after changing to a nonmoving fixture is valid data.
  p.lighting.fixtures[0].positionReference = {
    profileId:'19999999-0000-4000-8000-000000000001',
    profileRevision:'19999999-0000-4000-8000-000000000002',
    points:[{id:'19999999-0000-4000-8000-000000000003',name:'台口',
      targetMeters:{x:'0',y:'0',z:'0'},panValue:32768,tiltValue:32768,source:'sceneSetpoint'}],
  };
  return p;
}
test('模式已改变的旧参考记录仍可离线审查',()=>assert.doesNotThrow(()=>auditProject(document())));
for (const [name,mutate,pattern] of [
  ['能力缺失',p=>p.requires.pop(),/能力声明/],
  ['坐标越界',p=>p.lighting.fixtures[0].positionReference.points[0].targetMeters.x='100001',/坐标/],
  ['未知来源',p=>p.lighting.fixtures[0].positionReference.points[0].source='measured',/Schema/],
  ['重复对象',p=>p.lighting.fixtures[0].positionReference.points[0].id=p.lighting.fixtures[0].id,/重复/],
  ['无关字段',p=>p.lighting.fixtures[0].positionReference.points[0].offset=1,/Schema/],
]) test(`参考点拒绝${name}`,()=>{ const p=document();mutate(p);assert.throws(()=>auditProject(p),pattern); });
