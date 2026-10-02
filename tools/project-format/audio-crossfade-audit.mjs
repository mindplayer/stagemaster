import assert from 'node:assert/strict';
export function auditCrossfades(track, declared, get) {
  const clips = track?.lightingClips;
  const supported = declared.has('media.audio-clip-crossfade@1');
  assert(!supported || clips, '动态交叉能力缺少独立片段轨道');
  let values = (clips ?? []).reduce((n, c) => n + (c.entryFade?.from.length ?? 0), 0);
  for (const clip of clips ?? []) {
    if ((clip.fadeMode ?? 'snapshot') === 'snapshot') {
      assert(!clip.entryCrossfade, '保留交叉只能用于动态交叉模式');
      continue;
    }
    assert(supported, '缺少 media.audio-clip-crossfade 能力声明');
    assert(!clip.entryFade, '动态交叉不能同时具有静态保留渐变');
    if (!clip.fadeMs && !clip.entryCrossfade) continue;
    const previous = clips.find(p => p.enabled !== false && p.endMs === clip.startMs);
    const previousLength = previous ? previous.endMs - previous.startMs : 0;
    if (!clip.entryCrossfade && previous?.entryCrossfade) assert(previous.entryCrossfade.offsetMs + previousLength >= previous.entryCrossfade.durationMs, '前段动态交叉尚未结束');
    const fade = clip.entryCrossfade ?? {
      durationMs: clip.fadeMs,
      offsetMs: 0,
      source: {
        sceneId: previous?.sceneId ?? null,
        effectOffsetMs: previous?.effectOffsetMs ?? 0,
        elapsedMs: previousLength,
        entryFade: previous?.entryFade
      }
    };
    const {
        source
      } = fade,
      length = clip.endMs - clip.startMs;
    assert(fade.durationMs > 0 && fade.durationMs <= 3600000, '交叉总长无效');
    assert(fade.offsetMs + length <= 3600000, '交叉源范围超出一小时');
    assert(source.effectOffsetMs + source.elapsedMs + length <= 3600000, '交叉来源范围超出一小时');
    assert(clip.fadeMs === Math.min(Math.max(0, fade.durationMs - fade.offsetMs), length), '交叉可见长度不符');
    if (source.sceneId !== null) get(source.sceneId, 'scene');else assert(!source.entryFade && source.effectOffsetMs === 0, '默认值来源不能含场景渐变或效果偏移');
    const entry = source.entryFade;
    if (!entry) continue;
    assert(entry.offsetMs + source.elapsedMs + length <= 3600000, '来源进入渐变超出一小时');
    if (clip.entryCrossfade) values += entry.from.length;
    assert(values <= 32768, '保留渐变与交叉的起始值总数超过 32768');
    const keys = new Set();
    for (const value of entry.from) {
      const key = JSON.stringify([value.fixtureId, value.attribute]);
      assert(!keys.has(key), '交叉来源起始值重复');
      keys.add(key);
      const fixture = get(value.fixtureId, 'fixture');
      const attribute = get(fixture.profileId, 'profile').attributes.find(a => a.key === value.attribute);
      assert(attribute?.valueType.kind === 'normalized', '交叉来源引用的连续属性不存在');
    }
  }
}
