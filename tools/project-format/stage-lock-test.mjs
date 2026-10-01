import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function project() {
  const p=structuredClone(loadExamples().find(d=>d.project?.name==='单路灯光示例'));
  p.requires.push({key:'stage.spaces',version:1},{key:'stage.edit-locks',version:1});
  if(!p.requires.some(c=>c.key==='stage.layout'))p.requires.push({key:'stage.layout',version:1});
  p.stage={nodes:[],spaces:[{id:'a0000000-0000-4000-8000-000000000001',name:'空间',outlineMeters:[['0','0'],['4','0'],['4','4'],['0','4']],floorElevationMeters:'0',clearHeightMeters:'5'}],constructions:[],placements:[],editLocks:[{kind:'space',targetId:'a0000000-0000-4000-8000-000000000001'}]};
  return p;
}
test('锁定引用不创建新对象身份且能力严格声明',()=>{
  assert.doesNotThrow(()=>auditProject(project()));
  const p=project();p.requires=p.requires.filter(c=>c.key!=='stage.edit-locks');assert.throws(()=>auditProject(p),/锁定能力/);
});
test('锁定的悬空、重复、错类型、空及超限字段拒绝',()=>{
  for(const change of [p=>p.stage.editLocks[0].targetId='a0000000-0000-4000-8000-000000000002',p=>p.stage.editLocks.push({...p.stage.editLocks[0]}),p=>p.stage.editLocks[0].kind='placement',p=>p.stage.editLocks=[],p=>p.stage.editLocks[0].extra=true]) {
    const p=project();change(p);assert.throws(()=>auditProject(p));
  }
});
