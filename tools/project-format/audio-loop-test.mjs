import test from 'node:test';
import assert from 'node:assert/strict';
import { auditProject, loadExamples } from './check.mjs';

function project() {
  const p = structuredClone(loadExamples().find(p => p.project?.name === '单路灯光示例'));
  p.requires.push(...['media.audio-editing', 'media.audio-loop-regions']
    .map(key => ({key, version: 1})));
  p.media = {systems: [], objects: [], audioEditing: {
    asset: {digest: 'ab'.repeat(32), fileName: '演出.wav', extension: 'wav', durationMs: 30000},
    inMs: 1000, outMs: 30000, markers: [], loopRegions: [
      {id: 'd0000000-0000-4000-8000-000000000001', name: '等待演员',
        startMs: 1000, endMs: 3000, plays: {kind: 'untilExit'}, enabled: true, locked: false},
      {id: 'd0000000-0000-4000-8000-000000000002', name: '三次重复',
        startMs: 3000, endMs: 4000, plays: {kind: 'count', count: 3}, enabled: false, locked: true},
    ],
  }};
  return p;
}
const regions = p => p.media.audioEditing.loopRegions;

test('循环区段严格支持相邻、固定总次数、持续循环及停用锁定', () => {
  assert.doesNotThrow(() => auditProject(project()));
});

for (const [name, mutate] of [
  ['缺少能力', p => p.requires = p.requires.filter(r => r.key !== 'media.audio-loop-regions')],
  ['缺少音乐', p => delete p.media],
  ['重叠停用段', p => regions(p)[1].startMs = 2999],
  ['无序', p => regions(p).reverse()],
  ['越过裁切末端', p => regions(p)[1].endMs = 29001],
  ['空范围', p => regions(p)[0].endMs = 1000],
  ['重复身份', p => regions(p)[1].id = regions(p)[0].id],
  ['跨模块重复身份', p => regions(p)[1].id = p.lighting.fixtures[0].id],
  ['空白名称', p => regions(p)[1].name = '  '],
  ['零次', p => regions(p)[1].plays.count = 0],
  ['负数', p => regions(p)[1].plays.count = -1],
  ['小数', p => regions(p)[1].plays.count = 2.5],
  ['计数越界', p => regions(p)[1].plays.count = 4294967296],
  ['模糊的持续方式', p => regions(p)[0].plays = 'infinite'],
  ['持续方式附带次数', p => regions(p)[0].plays.count = 1],
  ['未知字段', p => regions(p)[0].repeat = true],
  ['缺少锁状态', p => delete regions(p)[0].locked],
  ['数组为空值', p => p.media.audioEditing.loopRegions = null],
  ['超容量', p => p.media.audioEditing.loopRegions = Array.from({length: 129}, (_, i) => ({
    ...regions(p)[0], id: `d0000000-0000-4000-8000-${String(i + 1).padStart(12, '0')}`,
    startMs: i * 2, endMs: i * 2 + 1,
  }))],
]) {
  test(`循环区段拒绝${name}`, () => {
    const p = project();
    mutate(p);
    assert.throws(() => auditProject(p));
  });
}

test('空区段及省略字段兼容旧工程，最大次数为合法有限次数', () => {
  const p = project();
  regions(p)[1].plays.count = 4294967295;
  assert.doesNotThrow(() => auditProject(p));
  p.media.audioEditing.loopRegions = [];
  assert.doesNotThrow(() => auditProject(p));
  delete p.media.audioEditing.loopRegions;
  p.requires = p.requires.filter(r => r.key !== 'media.audio-loop-regions');
  assert.doesNotThrow(() => auditProject(p));
});
