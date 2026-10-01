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
