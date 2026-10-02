import assert from 'node:assert/strict';

// Offline format audit only; runtime timing belongs to the Rust scheduler.
export function auditAudioLoops(track, declared, add) {
  const capability = declared.has('media.audio-loop-regions@1');
  assert(!capability || track, '循环区段能力声明缺少音乐轨道');
  const regions = track?.loopRegions ?? [];
  assert(!regions.length || capability, '缺少模块能力声明：media.audio-loop-regions');
  add('audio-loop-region', regions);
  let previousEnd = 0;
  for (const region of regions) {
    assert(region.name.trim(), '循环区段名称不能为空');
    assert(region.startMs >= previousEnd && region.startMs < region.endMs &&
      region.endMs <= track.outMs - track.inMs,
    '循环区段须有序、不重叠，且位于音乐裁切范围内');
    previousEnd = region.endMs;
  }
}
