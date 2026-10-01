import assert from 'node:assert/strict';
import { auditAudioClips } from './audio-clips-audit.mjs';

export function auditAudioEditing(project, declared, add, get) {
  const track = project.media?.audioEditing;
  if (!track) { assert(!declared.has('media.audio-clips@1'), '独立片段能力缺少音乐轨道'); return; }
  assert(declared.has('media.audio-editing@1'), '缺少模块能力声明：media.audio-editing');
  assert(track.inMs < track.outMs && track.outMs <= track.asset.durationMs, '音乐裁切范围无效');
  auditAudioClips(project, track, declared, add, get);
  add('audio-marker', track.markers);
  let previous = -1;
  for (const marker of track.markers) {
    assert(marker.timeMs > previous && marker.timeMs < track.outMs-track.inMs, '音乐卡点须有序、不重复且在裁切范围内');
    if (marker.sceneId) get(marker.sceneId, 'scene');
    const fade = marker.fadeMs ?? 0;
    if (fade) {
      assert(declared.has('media.audio-transitions@1'), '缺少模块能力声明：media.audio-transitions');
      const end = track.markers.find(next => next.sceneId && next.timeMs > marker.timeMs)?.timeMs ?? track.outMs-track.inMs;
      assert(marker.sceneId && fade <= end-marker.timeMs, '灯光渐变须绑定场景且不能超出段落');
    }
    previous = marker.timeMs;
  }
}
