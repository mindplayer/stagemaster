"""Build an editable curved stage from the user's seating-plan silhouette.

The supplied picture has no scale. Default dimensions are visual estimates,
not measured construction data. Reuses the existing concert's real fixtures,
patch, scenes and lists, without inserting anything into product code.
"""
import argparse
import copy
import json
import math
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def decimal(value):
    return f'{value:.6f}'.rstrip('0').rstrip('.') if value else '0'


def uid():
    return str(uuid.uuid4())


def build(source, width, depth, height, room_height=7):
    project = copy.deepcopy(source)
    stage = project['stage']
    old_space = stage['spaces'][0]['id']
    stage_space, auditorium = old_space, uid()
    ratio = depth / 6
    # Pixel landmarks of the orange silhouette, clockwise in the source image.
    # World Y increases toward the back wall, reversing the image's Y axis.
    pixels = [(271,68),(1099,68),(1115,88),(1128,118),(1135,149),
              (1131,181),(1114,215),(1080,250),(1025,286),(950,317),
              (864,340),(774,355),(684,362),(600,358),(506,346),
              (421,326),(350,299),(293,264),(255,221),(238,183),
              (235,152),(239,119),(249,93)]

    def point(x, y):
        return [(x - 235) / 900 * width, depth - (y - 68) / 294 * depth]

    outline = [[decimal(v) for v in point(x,y)] for x,y in pixels]
    stage['spaces'] = [
        {'id':stage_space,'name':'圆弧表演区','outlineMeters':outline,
         'floorElevationMeters':decimal(height),
         'clearHeightMeters':decimal(room_height-height)},
        {'id':auditorium,'name':'三面观众区','outlineMeters':[
            [decimal(-.2*width),decimal(-1.75*depth)],
            [decimal(1.3*width),decimal(-1.75*depth)],
            [decimal(1.3*width),decimal(depth)],
            [decimal(-.2*width),decimal(depth)]],
         'floorElevationMeters':'0','clearHeightMeters':decimal(room_height)},
    ]
    original = stage['constructions']
    stage['constructions'] = []

    def prism(name, points, base, tall, space=auditorium, identifier=None):
        stage['constructions'].append({'id':identifier or uid(),'name':name,'shape':{
            'kind':'platform','spaceId':space,
            'outlineMeters':[[decimal(x),decimal(y)] for x,y in points],
            'baseElevationMeters':decimal(base),'heightMeters':decimal(tall)}})

    prism('场地地面', [[-.2*width,-1.75*depth],[1.3*width,-1.75*depth],
                       [1.3*width,depth],[-.2*width,depth]], -.12,.12)
    prism('圆弧表演台', [point(x,y) for x,y in pixels],0,height,stage_space)
    prism('独立背墙', [[.04*width,depth-.08],[.96*width,depth-.08],
                      [.96*width,depth],[.04*width,depth]],height,room_height-height,stage_space)
    # Two side steps follow the actual side outline rather than covering the front arc.
    for side, x0, x1 in [('左',-.5,.12*width),('右',.88*width,width+.5)]:
        prism(side+'侧台阶',[[x0,depth*.65],[x1,depth*.65],
                             [x1,depth*.88],[x0,depth*.88]],0,height/2)
    for construction in original:
        if construction['shape']['kind'] != 'rig':
            continue
        shape = construction['shape']
        shape['positionMeters']['x'] = decimal(float(shape['positionMeters']['x']) * width / 10)
        shape['positionMeters']['y'] = decimal(float(shape['positionMeters']['y']) * ratio)
        shape['positionMeters']['z'] = decimal(float(shape['positionMeters']['z']) * (room_height-height)/6.5 + height)
        scale = ratio if float(shape['yawDegrees']) == 90 else width/10
        shape['lengthMeters'] = decimal(float(shape['lengthMeters']) * scale)
        stage['constructions'].append(construction)
    for placement in stage['placements']:
        position = placement['positionMeters']
        position['x'] = decimal(float(position['x']) * width/10)
        position['y'] = decimal(float(position['y']) * ratio)
        z=float(position['z'])
        position['z'] = decimal((z*(room_height-height)/6.5 if z>1 else z) + height)
    # Front edge PARs sit on the curved platform, not the old rectangular front edge.
    fixtures = {f['id']:f for f in project['lighting']['fixtures']}
    side_washes = 0
    for placement in stage['placements']:
        name = fixtures[placement['fixtureId']]['name']
        if '地排染色' in name:
            # Leave the central performance floor empty; two washes on each wing.
            left=side_washes<2
            placement['positionMeters']['x']=decimal(width*(.08 if left else .92))
            placement['positionMeters']['y']=decimal(depth*(.50 if side_washes%2==0 else .64))
            side_washes+=1
        if '前沿轮廓' in name:
            x = float(placement['positionMeters']['x'])
            # Intersection with the piecewise-linear front silhouette, plus inset.
            intersections = []
            points = [point(*pixel) for pixel in pixels]
            for a,b in zip(points, points[1:]+points[:1]):
                if min(a[0],b[0]) <= x <= max(a[0],b[0]) and a[0] != b[0]:
                    intersections.append(a[1]+(x-a[0])/(b[0]-a[0])*(b[1]-a[1]))
            placement['positionMeters']['y'] = decimal(min(intersections)+.22)
    # Remap the authored direction's horizontal projection into the shallower footprint.
    for scene in project['lighting']['scenes']:
        moving = {}
        for assignment in scene['assignments']:
            target = assignment['target']
            if target['attribute'] in ('pan','tilt'):
                moving.setdefault(target['fixtureId'],{})[target['attribute']] = assignment
        for axes in moving.values():
            pan = axes['pan']['source']['value']['value']/65535*540-270
            tilt = axes['tilt']['source']['value']['value']/65535*270-135
            angle = math.radians(pan)
            dx,dy = -math.sin(angle)*width/10, math.cos(angle)*ratio
            adjusted = math.degrees(math.atan2(-dx,dy))
            adjusted += round((pan-adjusted)/360)*360
            if adjusted>270: adjusted-=360
            if adjusted< -270: adjusted+=360
            tilt = math.degrees(math.atan(math.tan(math.radians(tilt))*math.hypot(dx,dy)))
            axes['pan']['source']['value']['value'] = round((adjusted+270)/540*65535)
            axes['tilt']['source']['value']['value'] = round((tilt+135)/270*65535)

    # Seat + backrest geometry is editable using existing platform primitives.
    # No reservation/customer/category semantics are inferred from the picture.
    def chair(zone, number, px, py, yaw=0):
        x,y=point(px,py)
        angle=math.radians(yaw)
        def rectangle(cx,cy,w,d):
            out=[]
            for u,v in [(cx-w/2,cy-d/2),(cx+w/2,cy-d/2),(cx+w/2,cy+d/2),(cx-w/2,cy+d/2)]:
                out.append([x+u*math.cos(angle)-v*math.sin(angle),
                            y+u*math.sin(angle)+v*math.cos(angle)])
            return out
        name=f'{zone}区 {number:02}'
        prism(name+' 座面',rectangle(0,0,.5,.45),.38,.08)
        prism(name+' 靠背',rectangle(0,-.205,.5,.05),.46,.38)
        # Two side supports keep the complete venue inside the 512 construction limit.
        for side,u in [('左',-.19),('右',.19)]:
            prism(name+side+'支脚',rectangle(u,0,.045,.36),0,.38)
    n=0
    for row in range(5):
        for column in range(6):
            n+=1;chair('A',n,532+65*column,425+58*row)
    for row, columns in [(0,range(16)),(1,range(3,12))]:
        for column in columns:
            n+=1;chair('A',n,148+65*column,768+68*row)
    # Angled blocks: 6/6/6/5 seats, traced from the supplied picture.
    for zone, mirror in [('C',False),('B',True)]:
        n=0
        for row in range(4):
            count=5 if row==3 else 6
            start_x,start_y=[(169,309),(136,366),(104,424),(118,513)][row]
            for column in range(count):
                px,py=start_x+52*column,start_y+30*column
                if mirror: px=1394-px
                n+=1;chair(zone,n,px,py,30 if mirror else -30)
    for sequence in project['lighting']['sequences']:
        sequence['name']=sequence['name'].replace('星河现场','圆弧剧场')
    project['project'].update(id=uid(),revisionId=uid(),parentRevisionIds=[],
        name='圆弧剧场 · 三面观众舞台',
        description=f'按用户座位图估算：圆弧表演台宽 {width:g} 米、进深 {depth:g} 米；用户指定台高 {height:g} 米、房间总高 {room_height:g} 米，台面以上净高 {room_height-height:g} 米。后沿直线、独立背墙，无侧墙和屋顶。A区55席、B/C区各23席；中央表演地面留空，落地灯沿边缘布置，悬挂灯和动态列表沿用并适配星河现场。宽深为未标尺图片的估算，可编辑调整；灯具仍是通用光学。')
    return project


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',type=Path,default=ROOT/'data/showcases/星河现场·10米舞台.project.json')
    parser.add_argument('--output',type=Path,default=ROOT/'data/showcases/圆弧剧场.project.json')
    parser.add_argument('--width',type=float,default=10)
    parser.add_argument('--depth',type=float,default=3.5)
    parser.add_argument('--height',type=float,default=.5)
    parser.add_argument('--room-height',type=float,default=7)
    args=parser.parse_args()
    if args.output.exists(): parser.error('目标工程已存在，拒绝覆盖用户编辑')
    if not all(math.isfinite(v) and v>0 for v in (args.width,args.depth,args.height,args.room_height)):
        parser.error('舞台尺寸须为有限正数')
    if args.room_height<=args.height:
        parser.error('房间总高须高于地台')
    project=build(json.loads(args.source.read_text()),args.width,args.depth,args.height,args.room_height)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(project,ensure_ascii=False,separators=(',',':'))+'\n')
    print(args.output)
