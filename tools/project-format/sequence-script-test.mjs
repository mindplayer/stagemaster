import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function project() {
  const p=structuredClone(loadExamples().find(d=>d.project?.name==='单路灯光示例'));
  p.requires.push({key:'lighting.sequence-script',version:1});
  p.lighting.sequences[0].steps[0].script={section:'第一幕',trigger:'演员举手\n执行',notes:'等掌声'};
  return p;
}
test('提示纯文本与能力、长度、控制字符严格检查',()=>{
  assert.doesNotThrow(()=>auditProject(project()));
  for(const change of [p=>p.requires.pop(),p=>p.lighting.sequences[0].steps[0].script.trigger='字'.repeat(1025),p=>p.lighting.sequences[0].steps[0].script.notes='\u0085']){
    const p=project();change(p);assert.throws(()=>auditProject(p));
  }
});
test('每列表剧本总容量受约束，不截断内容',()=>{
  const p=project(),seq=p.lighting.sequences[0],step=seq.steps[0];
  step.script.notes='字'.repeat(4096);
  for(let i=2;i<8;i++)seq.steps.push({...structuredClone(step),id:`a0000000-0000-4000-8000-${String(i).padStart(12,'0')}`,number:String(i+1)});
  assert.throws(()=>auditProject(p),/64 KiB/);
});
