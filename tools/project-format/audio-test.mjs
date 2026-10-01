import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function project() {
  const p = structuredClone(loadExamples().find(d => d.project?.name === '单路灯光示例'));
  p.requires.push({key:'media.audio-editing',version:1});
  p.media = {systems:[], objects:[], audioEditing:{
    asset:{digest:'ab'.repeat(32),fileName:'音乐.mp3',extension:'mp3',durationMs:30000},
    inMs:0,outMs:30000,markers:[{id:'a0000000-0000-4000-8000-000000000001',
      name:'起势',timeMs:1000,sceneId:p.lighting.scenes[0].id}]}};
  return p;
}
test('本机音乐不被误判为外部媒体模块',()=>assert.doesNotThrow(()=>auditProject(project())));
test('音乐必须有专用能力声明',()=>{
  const p=project();p.requires=p.requires.filter(v=>v.key!=='media.audio-editing');
  assert.throws(()=>auditProject(p),/media.audio-editing/);
});
test('音乐卡点引用、裁切范围和重复时间均受检查',()=>{
  for (const mutate of [
    p=>p.media.audioEditing.markers[0].sceneId=p.lighting.fixtures[0].id,
    p=>p.media.audioEditing.inMs=30000,
    p=>p.media.audioEditing.markers.push({...p.media.audioEditing.markers[0],id:'a0000000-0000-4000-8000-000000000002'}),
    p=>p.media.audioEditing.markers[0].timeMs=30000,
  ]) {const p=project();mutate(p);assert.throws(()=>auditProject(p));}
});
test('外部媒体仍须声明自己的能力',()=>{
  const p=structuredClone(loadExamples().find(d=>d.project?.name==='密室声光电联动示例'));
  p.requires=p.requires.filter(v=>v.key!=='media.external');
  assert.throws(()=>auditProject(p),/media.external/);
});

test('音乐灯光渐变有独立能力、段落边界和零渐变兼容',()=>{
  const p=project();const marker=p.media.audioEditing.markers[0];
  marker.fadeMs=0;assert.doesNotThrow(()=>auditProject(p));
  marker.fadeMs=1000;assert.throws(()=>auditProject(p),/media.audio-transitions/);
  p.requires.push({key:'media.audio-transitions',version:1});
  assert.doesNotThrow(()=>auditProject(p));
  marker.fadeMs=29001;assert.throws(()=>auditProject(p),/渐变/);
  marker.fadeMs=1000;marker.sceneId=null;assert.throws(()=>auditProject(p),/渐变/);
});
test('纯节奏标记不截断渐变，绑定场景才形成下一边界',()=>{
  const p=project();p.requires.push({key:'media.audio-transitions',version:1});
  p.media.audioEditing.markers[0].fadeMs=2000;
  const next={id:'a0000000-0000-4000-8000-000000000002',name:'中间拍',timeMs:2000,sceneId:null};
  p.media.audioEditing.markers.push(next);assert.doesNotThrow(()=>auditProject(p));
  next.sceneId=p.lighting.scenes[0].id;assert.throws(()=>auditProject(p),/渐变/);
});

function clipProject() {
  const p=project();p.requires.push({key:'media.audio-clips',version:1});
  p.media.audioEditing.markers[0].sceneId=null;
  p.media.audioEditing.lightingClips=[{id:'d0000000-0000-4000-8000-000000000001',name:'片段',sceneId:p.lighting.scenes[0].id,startMs:1000,endMs:3000,fadeMs:1000,locked:false}];
  return p;
}
test('独立灯光片段允许空隙、恰好相邻和独立锁定',()=>{
  const p=clipProject();p.media.audioEditing.lightingClips.push({...p.media.audioEditing.lightingClips[0],id:'d0000000-0000-4000-8000-000000000002',startMs:3000,endMs:5000,locked:true});
  assert.doesNotThrow(()=>auditProject(p));
});
test('片段能力、区间、引用、身份、容量与双调度分别拒绝',()=>{
  for(const mutate of [
    p=>p.requires=p.requires.filter(r=>r.key!=='media.audio-clips'),
    p=>delete p.media.audioEditing.lightingClips,
    p=>p.media.audioEditing.markers[0].sceneId=p.lighting.scenes[0].id,
    p=>p.media.audioEditing.lightingClips[0].endMs=1000,
    p=>p.media.audioEditing.lightingClips[0].endMs=30001,
    p=>p.media.audioEditing.lightingClips[0].fadeMs=2001,
    p=>p.media.audioEditing.lightingClips[0].sceneId=p.lighting.fixtures[0].id,
    p=>p.media.audioEditing.lightingClips[0].id=p.media.audioEditing.markers[0].id,
    p=>p.media.audioEditing.lightingClips[0].locked='yes',
    p=>p.media.audioEditing.lightingClips.push({...p.media.audioEditing.lightingClips[0],id:'d0000000-0000-4000-8000-000000000002',startMs:2500}),
    p=>p.media.audioEditing.lightingClips=Array.from({length:513},(_,i)=>({...p.media.audioEditing.lightingClips[0],id:`d0000000-0000-4000-8000-${String(i).padStart(12,'0')}`,startMs:i*10,endMs:i*10+5,fadeMs:0})),
  ]) {const p=clipProject();mutate(p);assert.throws(()=>auditProject(p));}
});

test('片段启停默认兼容，停用需要能力且保留区间约束',()=>{
  const p=clipProject();const c=p.media.audioEditing.lightingClips[0];
  c.enabled=true;assert.doesNotThrow(()=>auditProject(p));
  c.enabled=false;assert.throws(()=>auditProject(p),/media.audio-clip-state/);
  p.requires.push({key:'media.audio-clip-state',version:1});assert.doesNotThrow(()=>auditProject(p));
  c.enabled='false';assert.throws(()=>auditProject(p));c.enabled=false;
  c.endMs=c.startMs;assert.throws(()=>auditProject(p));
});
test('片段启停能力不允许缺失轨道',()=>{
  for(const mutate of [p=>delete p.media,p=>delete p.media.audioEditing.lightingClips]){
    const p=clipProject();p.requires.push({key:'media.audio-clip-state',version:1});mutate(p);assert.throws(()=>auditProject(p));
  }
});


test('片段效果起点默认兼容且须声明能力，并限制源时间范围',()=>{
  const p=clipProject();const c=p.media.audioEditing.lightingClips[0];
  c.effectOffsetMs=0;assert.doesNotThrow(()=>auditProject(p));
  c.effectOffsetMs=333;assert.throws(()=>auditProject(p),/media.audio-clip-offset/);
  p.requires.push({key:'media.audio-clip-offset',version:1});assert.doesNotThrow(()=>auditProject(p));
  for(const value of [-1,1.5,'333',null,3600001,3599000]) {c.effectOffsetMs=value;assert.throws(()=>auditProject(p));}
});
test('效果起点能力必须有独立灯光片段轨道',()=>{
  for(const mutate of [p=>delete p.media,p=>delete p.media.audioEditing.lightingClips]){
    const p=clipProject();p.requires.push({key:'media.audio-clip-offset',version:1});mutate(p);assert.throws(()=>auditProject(p));
  }
});

test('孤立的效果起点能力独立拒绝，不依靠其他片段能力检查',()=>{
  const p=structuredClone(loadExamples().find(d => d.project?.name === '单路灯光示例'));
  p.requires.push({key:'media.audio-clip-offset',version:1});
  assert.throws(()=>auditProject(p),/缺少音乐轨道/);
});
