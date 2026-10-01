import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function project() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  p.requires.push(...['media.audio-editing','media.audio-clips','media.audio-clip-fade'].map(key => ({key,version:1})));
  p.media = {systems:[],objects:[],audioEditing:{asset:{digest:'ab'.repeat(32),fileName:'音乐.wav',extension:'wav',durationMs:30000},inMs:0,outMs:30000,markers:[],lightingClips:[{
    id:'d0000000-0000-4000-8000-000000000001',name:'片段',sceneId:p.lighting.scenes[0].id,startMs:1000,endMs:2000,fadeMs:250,locked:false,
    entryFade:{durationMs:500,offsetMs:250,from:[{fixtureId:p.lighting.fixtures[0].id,attribute:'dimmer',value:40000}]}
  }]}};
  return p;
}
const clip = p => p.media.audioEditing.lightingClips[0];
test('历史渐变保持独立时基和稳定属性引用', () => assert.doesNotThrow(() => auditProject(project())));
for(const [name, mutate] of [
  ['缺少能力', p => p.requires=p.requires.filter(r => r.key !== 'media.audio-clip-fade')],
  ['缺少轨道', p => delete p.media.audioEditing.lightingClips],
  ['缺少音乐', p => delete p.media],
  ['未知字段', p => clip(p).entryFade.address=1],
  ['错误时间类型', p => clip(p).entryFade.offsetMs='250'],
  ['渐变总长为零', p => clip(p).entryFade.durationMs=0],
  ['可见长度不符', p => clip(p).fadeMs=251],
  ['源范围越界', p => {clip(p).entryFade.offsetMs=3599999;clip(p).fadeMs=0;}],
  ['重复属性', p => clip(p).entryFade.from.push({...clip(p).entryFade.from[0]})],
  ['错误身份', p => clip(p).entryFade.from[0].fixtureId=p.project.id],
  ['未知属性', p => clip(p).entryFade.from[0].attribute='missing'],
  ['值越界', p => clip(p).entryFade.from[0].value=65536],
]) test(`历史渐变拒绝${name}`, () => {const p=project();mutate(p);assert.throws(() => auditProject(p));});
test('渐变结束后仍保留源时间以支持向左恢复', () => {
  const p=project();clip(p).entryFade.offsetMs=900;clip(p).fadeMs=0;assert.doesNotThrow(() => auditProject(p));
});
