import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {auditProject,loadExamples} from './check.mjs';
const vector=JSON.parse(readFileSync(new URL('../test-data/seating-layout.json',import.meta.url),'utf8'));
function project() {
  const p=structuredClone(loadExamples().find(d=>d.project?.name==='单路灯光示例'));
  for(const key of ['stage.layout','stage.spaces','stage.seating']) if(!p.requires.some(c=>c.key===key))p.requires.push({key,version:1});
  p.stage={nodes:[],spaces:[],placements:[],constructions:[{id:'a0000000-0000-4000-8000-000000000001',name:'座区',shape:structuredClone(vector.shape)}]};return p;
}
test('座区声明、间距、整数、未知字段和世界边界检查',()=>{
  assert.doesNotThrow(()=>auditProject(project()));
  for(const change of [p=>p.requires=p.requires.filter(c=>c.key!=='stage.seating'),p=>p.stage.constructions[0].shape.rows=1.5,p=>p.stage.constructions[0].shape.extra=true,p=>p.stage.constructions[0].shape.rowSpacingMeters='.4',p=>p.stage.constructions[0].shape.aisle.widthMeters='.3',p=>p.stage.constructions[0].shape.positionMeters.z='100000']) { const p=project();change(p);assert.throws(()=>auditProject(p)); }
});
test('座区和工程分别限制座位数量',()=>{
  const p=project(),c=p.stage.constructions[0];c.shape.rows=64;c.shape.columns=8;assert.doesNotThrow(()=>auditProject(p));
  p.stage.constructions.push({...structuredClone(c),id:'a0000000-0000-4000-8000-000000000002'});assert.doesNotThrow(()=>auditProject(p));
  p.stage.constructions.push({...structuredClone(c),id:'a0000000-0000-4000-8000-000000000003'});assert.throws(()=>auditProject(p),/1024/);
  c.shape.columns=9;assert.throws(()=>auditProject(p),/512/);
});

test('弧排独立三角向量、能力、净宽与重叠拒绝',()=>{
  const vector=JSON.parse(readFileSync(new URL('../test-data/seating-arc-layout.json',import.meta.url),'utf8'));
  const good=project();good.stage.constructions[0].shape=vector.shape;good.requires.push({key:'stage.seating.arc',version:1});
  assert.doesNotThrow(()=>auditProject(good));
  for(const change of [p=>p.requires=p.requires.filter(c=>c.key!=='stage.seating.arc'),p=>p.stage.constructions[0].shape.arc.radiusMeters='1',p=>p.stage.constructions[0].shape.columnSpacingMeters='0.5',p=>p.stage.constructions[0].shape.arc.extra=true]) {const p=structuredClone(good);change(p);assert.throws(()=>auditProject(p));}
  good.stage.constructions[0].shape.columns=4;good.stage.constructions[0].shape.columnSpacingMeters='0.6';good.stage.constructions[0].shape.aisle={afterColumn:2,widthMeters:'1.2'};assert.doesNotThrow(()=>auditProject(good));
});
