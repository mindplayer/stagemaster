import assert from 'node:assert/strict';

export function auditAudioEditing(project, declared, add, get) {
  const track = project.media?.audioEditing;
  if (!track) return;
  assert(declared.has('media.audio-editing@1'), '缺少模块能力声明：media.audio-editing');
  assert(track.inMs < track.outMs && track.outMs <= track.asset.durationMs, '音乐裁切范围无效');
  add('audio-marker', track.markers);
  let previous = -1;
  for (const marker of track.markers) {
    assert(marker.timeMs > previous && marker.timeMs < track.outMs-track.inMs, '音乐卡点须有序、不重复且在裁切范围内');
    if (marker.sceneId) get(marker.sceneId, 'scene');
    previous = marker.timeMs;
  }
}
