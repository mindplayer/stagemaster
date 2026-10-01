// Archived setpoint references stay valid data after changing a fixture mode.
export function auditPositionReferences(project, declared, add) {
  let total = 0;
  for (const fixture of project.lighting?.fixtures ?? []) {
    const record = fixture.positionReference;
    if (!record) continue;
    if (!declared.has('lighting.position-reference@1')) throw new Error('参考点缺少位置参考能力声明');
    total += record.points.length;
    if (total > 1024) throw new Error('工程参考点总数不能超过 1024');
    add('position-reference-point', record.points);
    const names = new Set();
    for (const point of record.points) {
      const name = point.name.trim();
      if (!name || names.has(name)) throw new Error('参考点名称为空或重复');
      names.add(name);
      if (Object.values(point.targetMeters).some(v => !Number.isFinite(Number(v)) || Math.abs(Number(v)) > 100000)) {
        throw new Error('参考点世界坐标超出范围');
      }
    }
  }
}
