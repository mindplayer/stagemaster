import { createPortal } from "react-dom";
import { useEffect, useRef, useState } from 'react';
import type { StageEdit } from '../../stage-types';
import { decimal, rectangle } from '../../stage-tools';
import { validateEditorForm } from '../workbench/form-validation';
import { lShape } from '../../stage-geometry';

export type Creation = { kind: 'space' | 'platform'; name: string; x: number; y: number; elevation: string; spaceId: string | null };
export function StageCreateDialog({ initial, busy, error, onCreate, onCancel }: {
  initial: Creation; busy: boolean; error: string;
  onCreate(command: StageEdit): void; onCancel(): void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [problem, setProblem] = useState("");
  const [shape, setShape] = useState<'rectangle' | 'l'>('rectangle');
  const [name, setName] = useState(initial.name);
  const [width, setWidth] = useState(initial.kind === 'space' ? '8' : '4');
  const [depth, setDepth] = useState(initial.kind === 'space' ? '6' : '2');
  const [height, setHeight] = useState(initial.kind === 'space' ? '5' : '0.6');
  const [elevation, setElevation] = useState(initial.elevation);
  const [notchWidth, setNotchWidth] = useState('2'), [notchDepth, setNotchDepth] = useState('2');
  const [indoor, setIndoor] = useState(true);
  useEffect(() => { dialog.current?.showModal(); }, []);
  const w = Number(width), d = Number(depth);
  let points: [string, string][] = [];
  try { if (w >= .01 && w <= 10000 && d >= .01 && d <= 10000) points = shape === 'l' ? lShape(0, 0, w, d, Number(notchWidth), Number(notchDepth)) : rectangle(0, 0, w, d); } catch { /* invalid preview stays empty */ }
  const field = (label: string, value: string, change: (v: string) => void, min = 0.01, max = 10000) => <label>{label}<input aria-label={label} type="number" step="any" min={min} max={max} required value={value} onChange={e => change(e.target.value)} /></label>;
  return createPortal(<dialog ref={dialog} aria-modal="true" className="wb-dialog stage-create-dialog" aria-labelledby="stage-create-title" onCancel={e => { e.preventDefault(); if (!busy) onCancel(); }}>
    <form noValidate onSubmit={e => {
      e.preventDefault();
      try { validateEditorForm(e.currentTarget); } catch (reason) { setProblem(String(reason instanceof Error ? reason.message : reason)); return; }
      setProblem("");
      const outlineMeters = points.map(p => [decimal(Number(p[0]) + initial.x), decimal(Number(p[1]) + initial.y)] as [string, string]);
      if (outlineMeters.length < 3) return;
      onCreate(initial.kind === 'space'
        ? { op: 'putSpace', id: null, name: name.trim(), outlineMeters, floorElevationMeters: elevation, clearHeightMeters: indoor ? height : null }
        : { op: 'putConstruction', id: null, name: name.trim(), shape: { kind: 'platform', spaceId: initial.spaceId, outlineMeters, baseElevationMeters: elevation, heightMeters: height } });
    }}>
      <h2 id="stage-create-title">{initial.kind === 'space' ? '新建空间' : '新建舞台'}</h2>
      <fieldset disabled={busy}>
        <label>名称<input autoFocus required maxLength={120} aria-label="新对象名称" value={name} onChange={e => setName(e.target.value)} /></label>
        <div className="stage-shapes" aria-label="轮廓形状">
          <button type="button" aria-pressed={shape === 'rectangle'} onClick={() => setShape('rectangle')}>▭ 矩形</button>
          <button type="button" aria-pressed={shape === 'l'} onClick={() => { setShape('l'); setNotchWidth(decimal(w > 0 ? w / 3 : 2)); setNotchDepth(decimal(d > 0 ? d / 3 : 2)); }}>└ L 形</button>
        </div>
        <div className="stage-shape-preview">
          {w > 0 && d > 0 && points.length > 0 && <svg aria-label="新对象轮廓预览" viewBox={`-1 -1 ${w + 2} ${d + 2}`}><polygon points={points.map(p => `${p[0]},${d - Number(p[1])}`).join(' ')} /></svg>}
          <span>{width || '—'} × {depth || '—'} 米</span>
        </div>
        <div className="stage-pair">{field('宽度（米）', width, setWidth)}{field('深度（米）', depth, setDepth)}</div>
        {shape === 'l' && <div className="stage-pair">{field('右上缺口宽（米）', notchWidth, setNotchWidth, 0.01, Math.max(0, w - 0.01))}{field('右上缺口深（米）', notchDepth, setNotchDepth, 0.01, Math.max(0, d - 0.01))}</div>}
        {initial.kind === 'space' && <label className="stage-check"><input type="checkbox" checked={indoor} onChange={e => setIndoor(e.target.checked)} />室内空间</label>}
        <div className="stage-pair">{field(initial.kind === 'space' ? '地面标高（米）' : '基底标高（米）', elevation, setElevation, -10000)}{(indoor || initial.kind === 'platform') && field(initial.kind === 'space' ? '净高（米）' : '舞台高度（米）', height, setHeight)}</div>
      </fieldset>
      {(problem || error) && <p className="stage-error" role="alert">{problem || error}</p>}
      <div className="wb-dialog-actions"><button type="button" disabled={busy} onClick={onCancel}>取消</button><button className="wb-primary" disabled={busy} type="submit">创建</button></div>
    </form>
  </dialog>, document.querySelector(".workbench") ?? document.body);
}
