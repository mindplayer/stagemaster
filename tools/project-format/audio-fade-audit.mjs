import assert from 'node:assert/strict';
export function auditClipFades(track, declared, get) {
  const clips = track?.lightingClips;
  const supported = declared.has('media.audio-clip-fade@1');
  assert(!supported || clips, '保留渐变能力缺少独立片段轨道');
  let count = 0;
  for (const clip of clips ?? []) {
    const fade = clip.entryFade;
    if (!fade) continue;
    assert(supported, '缺少 media.audio-clip-fade 能力声明');
    assert(fade.offsetMs + clip.endMs - clip.startMs <= 3600000, '保留渐变源范围超过一小时');
    assert(clip.fadeMs === Math.min(Math.max(0, fade.durationMs - fade.offsetMs), clip.endMs - clip.startMs), '保留渐变与可见长度不符');
    count += fade.from.length;
    assert(count <= 32768, '保留渐变的起始值总数超过 32768');
    const targets = new Set();
    for (const value of fade.from) {
      const key = JSON.stringify([value.fixtureId, value.attribute]);
      assert(!targets.has(key), '保留渐变起始值属性重复'); targets.add(key);
      const fixture = get(value.fixtureId, 'fixture');
      const attribute = get(fixture.profileId, 'profile').attributes.find(attribute => attribute.key === value.attribute);
      assert(attribute?.valueType.kind === 'normalized', '保留渐变引用的连续属性不存在');
    }
  }
}
