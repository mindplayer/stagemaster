export function auditStageLocks(project) {
  const locks = project.stage?.editLocks ?? [];
  if (locks.length && !project.requires.some(c => c.key === 'stage.edit-locks' && c.version === 1)) throw new Error('缺少场地锁定能力声明');
  const objects = new Set([
    ...(project.stage?.spaces ?? []).map(s => `space:${s.id}`),
    ...(project.stage?.constructions ?? []).map(s => `construction:${s.id}`),
    ...(project.stage?.placements ?? []).map(s => `placement:${s.fixtureId}`),
  ]);
  const seen = new Set();
  for (const lock of locks) {
    const key = `${lock.kind}:${lock.targetId}`;
    if (!objects.has(key)) throw new Error('场地锁定对象不存在');
    if (seen.has(key)) throw new Error('场地锁定对象重复');
    seen.add(key);
  }
}
