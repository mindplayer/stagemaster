import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';
function project() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  p.requires.push(...['media.audio-editing', 'media.audio-clips', 'media.audio-clip-crossfade'].map(key => ({
    key,
    version: 1
  })));
  p.media = {
    systems: [],
    objects: [],
    audioEditing: {
      asset: {
        digest: 'ab'.repeat(32),
        fileName: '曲.wav',
        extension: 'wav',
        durationMs: 30000
      },
      inMs: 0,
      outMs: 30000,
      markers: [],
      lightingClips: [{
        id: 'd0000000-0000-4000-8000-000000000001',
        name: '动态交叉',
        sceneId: p.lighting.scenes[0].id,
        startMs: 1000,
        endMs: 2000,
        fadeMs: 250,
        fadeMode: 'dynamic',
        locked: false,
        entryCrossfade: {
          durationMs: 500,
          offsetMs: 250,
          source: {
            sceneId: p.lighting.scenes[0].id,
            effectOffsetMs: 100,
            elapsedMs: 500,
            entryFade: {
              durationMs: 800,
              offsetMs: 0,
              from: [{
                fixtureId: p.lighting.fixtures[0].id,
                attribute: 'dimmer',
                value: 12345
              }]
            }
          }
        }
      }]
    }
  };
  return p;
}
const clip = p => p.media.audioEditing.lightingClips[0];
const fade = p => clip(p).entryCrossfade;
test('动态来源支持独立历史渐变且保持语义身份', () => assert.doesNotThrow(() => auditProject(project())));
for (const [name, mutate] of [['缺少能力', p => p.requires = p.requires.filter(r => r.key !== 'media.audio-clip-crossfade')], ['缺少轨道', p => delete p.media.audioEditing.lightingClips], ['缺少音乐', p => delete p.media], ['静态模式混用', p => clip(p).fadeMode = 'snapshot'], ['静态历史混用', p => clip(p).entryFade = fade(p).source.entryFade], ['交叉嵌套', p => fade(p).source.entryCrossfade = {}], ['未知来源', p => fade(p).source.sceneId = p.project.id], ['来源时间类型错误', p => fade(p).source.elapsedMs = '1'], ['来源范围溢出', p => fade(p).source.effectOffsetMs = 3600000], ['交叉源范围溢出', p => {
  fade(p).offsetMs = 3600000;
  clip(p).fadeMs = 0;
}], ['来源渐变范围溢出', p => fade(p).source.entryFade.offsetMs = 3600000], ['无来源却有场景渐变', p => fade(p).source.sceneId = null], ['重复起始值', p => fade(p).source.entryFade.from.push({
  ...fade(p).source.entryFade.from[0]
})], ['起始属性缺失', p => fade(p).source.entryFade.from[0].attribute = 'missing'], ['可见长度错误', p => clip(p).fadeMs = 200], ['总长为零', p => fade(p).durationMs = 0]]) test(`动态交叉拒绝${name}`, () => {
  const p = project();
  mutate(p);
  assert.throws(() => auditProject(p));
});
test('相邻第三源拒绝，沿用同一历史的分割允许', () => {
  const p = project(),
    first = clip(p);
  first.endMs = 1100;
  first.fadeMs = 100;
  const second = {
    ...structuredClone(first),
    id: 'd0000000-0000-4000-8000-000000000002',
    startMs: 1100,
    endMs: 2000,
    fadeMs: 150
  };
  second.entryCrossfade.offsetMs += 100;
  second.entryCrossfade.source.elapsedMs += 100;
  p.media.audioEditing.lightingClips.push(second);
  assert.doesNotThrow(() => auditProject(p));
  delete second.entryCrossfade;
  assert.throws(() => auditProject(p));
});
