import { curvedSeatingBounds } from "./seating-arc-audit.mjs";
// Offline bounds/quantity audit; Rust remains the authoritative geometry implementation.
export function auditSeating(project) {
  const sections = (project.stage?.constructions ?? []).filter(c => c.shape.kind === 'seating');
  const assert = (condition, message) => { if (!condition) throw new Error(message); };
  if (sections.length) assert(project.requires.some(c => c.key === 'stage.seating' && c.version === 1), '座区缺少能力声明');
  let count = 0;
  for (const {shape: s} of sections) {
    assert(Number.isInteger(s.rows) && Number.isInteger(s.columns) && s.rows >= 1 && s.columns >= 1 && s.rows <= 64 && s.columns <= 64, '座区排列数越界');
    assert(s.rows * s.columns <= 512, '单座区最多 512 座');
    count += s.rows * s.columns;
    assert(count <= 1024, '工程最多 1024 座');
    const number = (v, min, max) => {
      const n = Number(v);
      assert(typeof v === 'string' && v.trim() !== '' && Number.isFinite(n) && n >= min && n <= max, '座区尺寸越界');
      return n;
    };
    const x = number(s.positionMeters.x, -100000, 100000), y = number(s.positionMeters.y, -100000, 100000);
    const z = number(s.positionMeters.z, -100000, 100000);
    const yaw = number(s.yawDegrees, -3600, 3600) * Math.PI / 180;
    const w = number(s.seatWidthMeters, .3, 1.2), d = number(s.seatDepthMeters, .3, 1.2);
    const dx = number(s.columnSpacingMeters, w, 5), dy = number(s.rowSpacingMeters, d, 10);
    let extra = 0;
    if (s.aisle) {
      assert(Number.isInteger(s.aisle.afterColumn) && s.aisle.afterColumn >= 1 && s.aisle.afterColumn < s.columns, '通道位置越界');
      const width = number(s.aisle.widthMeters, .3, 10);
      assert(width + 1e-9 >= dx - w, '通道净宽过小');
      extra = Math.max(0, width - (dx - w));
    }
    let halfW = ((s.columns - 1) * dx + w + extra) / 2, halfD = ((s.rows - 1) * dy + d) / 2;
    if (s.arc) {
      assert(project.requires.some(c=>c.key==='stage.seating.arc'&&c.version===1),'弧排缺少能力声明');
      [halfW,halfD]=curvedSeatingBounds(s,w,d,dx,dy,assert);
    }
    for (const a of [-halfW, halfW]) for (const b of [-halfD, halfD]) {
      assert(Math.abs(x + a * Math.cos(yaw) - b * Math.sin(yaw)) <= 100000 && Math.abs(y + a * Math.sin(yaw) + b * Math.cos(yaw)) <= 100000 && z + .85 <= 100000, '座区边界越界');
    }
  }
}
