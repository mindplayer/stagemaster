// Authoring metadata validation, never a playback interpreter.
export function auditSequenceScripts(project) {
  const declared = project.requires.some(c => c.key === 'lighting.sequence-script' && c.version === 1);
  for (const sequence of project.lighting?.sequences ?? []) {
    let bytes = 0;
    for (const step of sequence.steps) {
      if (!step.script) continue;
      if (!declared) throw new Error('缺少剧本提示能力声明');
      for (const value of Object.values(step.script)) {
        if (/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/.test(value)) throw new Error('剧本提示不能包含不可见控制字符');
        bytes += Buffer.byteLength(value, 'utf8');
      }
    }
    if (bytes > 64 * 1024) throw new Error('列表剧本提示超过 64 KiB');
  }
}
