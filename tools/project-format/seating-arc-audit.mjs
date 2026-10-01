// Independent review geometry: binary search for clear aisle and polygon SAT.
export function curvedSeatingBounds(s, w, d, dx, dy, assert) {
  const r=Number(s.arc.radiusMeters);
  assert(Number.isFinite(r)&&r>=1&&r<=10000,'弧排半径越界');
  const step=dx/r;
  let gap=0;
  if(s.aisle) {
    const width=Number(s.aisle.widthMeters);
    const clear=angle=>(2*r-d)*Math.sin(angle/2)-w*Math.cos(angle/2);
    assert(clear(Math.PI)>=width,'半径不足以容纳通道');
    let lo=0,hi=Math.PI;
    for(let i=0;i<60;i++){const mid=(lo+hi)/2;if(clear(mid)<width)lo=mid;else hi=mid;}
    gap=Math.max(0,(lo+hi)/2-step);
  }
  const spread=(s.columns-1)*step+gap;
  assert(spread+2*Math.atan(w/(2*r-d))<=Math.PI+1e-9,'弧排超过半圆');
  const chairs=[],centers=[];
  for(let row=0;row<s.rows;row++)for(let col=0;col<s.columns;col++) {
    const angle=-spread/2+col*step+(s.aisle&&col>=s.aisle.afterColumn?gap:0);
    const radius=r+row*dy,sin=Math.sin(angle),cos=Math.cos(angle);
    const center=[radius*sin,r-radius*cos];centers.push(center);
    chairs.push([[-w/2,-d/2],[w/2,-d/2],[w/2,d/2],[-w/2,d/2]].map(([x,y])=>[center[0]+x*cos-y*sin,center[1]+x*sin+y*cos]));
  }
  for(let i=0;i<chairs.length;i++)for(let j=0;j<i;j++) {
    if(Math.hypot(centers[i][0]-centers[j][0],centers[i][1]-centers[j][1])>=Math.hypot(w,d))continue;
    const separated=[chairs[i],chairs[j]].some(p=>[0,1].some(k=>{
      const x=p[k+1][0]-p[k][0],y=p[k+1][1]-p[k][1],len=Math.hypot(x,y);
      const interval=c=>{const values=c.map(q=>(q[0]*x+q[1]*y)/len);return [Math.min(...values),Math.max(...values)];};
      const a=interval(chairs[i]),b=interval(chairs[j]);return a[1]<=b[0]+1e-9||b[1]<=a[0]+1e-9;
    }));
    assert(separated,'弧排座椅重叠');
  }
  const points=chairs.flat(),xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
  return [(Math.max(...xs)-Math.min(...xs))/2,(Math.max(...ys)-Math.min(...ys))/2];
}
