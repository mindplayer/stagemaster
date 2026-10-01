import assert from 'node:assert/strict';
export function auditAudioClips(project, track, declared, add, get) {
  const clips = track.lightingClips;
  assert((clips !== undefined) === declared.has('media.audio-clips@1'), '独立灯光片段须与 media.audio-clips 能力同时存在');
  if (clips === undefined) return;
  add('audio-lighting-clip', clips);
  assert(track.markers.every(m => !m.sceneId && !(m.fadeMs ?? 0)), '片段模式卡点只能作节奏标记');
  let previousEnd = 0;
  for (const clip of clips) {
    assert(clip.name.trim(), '片段名称不能为空');
    assert(clip.startMs >= previousEnd && clip.startMs < clip.endMs && clip.endMs <= track.outMs-track.inMs, '灯光片段必须有序、无重叠且在音乐内');
    assert(clip.fadeMs <= clip.endMs-clip.startMs, '片段渐变不能超过长度');
    get(clip.sceneId, 'scene');
    previousEnd = clip.endMs;
  }
}
