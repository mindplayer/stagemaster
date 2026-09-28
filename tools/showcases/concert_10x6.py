"""Generate an editable StageMaster concert project, without device output.

Layout follows front/top/back/side functions (ETC lighting angles guide).
Fixture profiles are generic RGBD / 16-bit pan-tilt, not manufacturer fixtures.
Motion is authored as real sequence fades; it is not an unimplemented phaser.
"""
import copy
import json
import math
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / 'data/showcases/星河现场·10米舞台.project.json'
if DEST.exists():
 raise SystemExit('工程文件已存在；请先在应用内另存或明确移动原文件，避免覆盖用户编辑。')
IDS = {}
def uid(name):
 if name not in IDS: IDS[name] = str(uuid.uuid4())
 return IDS[name]
def dec(n): return f'{n:.6f}'.rstrip('0').rstrip('.') if n else '0'
def vec(x,y,z): return dict(zip(('x','y','z'),map(dec,(x,y,z))))
def norm(v): return round(max(0,min(1,v))*65535)
def dur(ms): return {'ticks':str(ms),'ticksPerSecond':'1000'}
space, domain = uid('space'), uid('domain')
lighting = {k:[] for k in ('profiles','fixtures','patches','groups','presets','scenes','sequences')}
stage = {k:[] for k in ('nodes','spaces','constructions','placements','attachments')}
stage['spaces']=[{'id':space,'name':'主舞台 · 10 × 6 × 6.5 米','outlineMeters':[['0','0'],['10','0'],['10','6'],['0','6']],'floorElevationMeters':'0','clearHeightMeters':'6.5'}]
def prism(name,x0,y0,x1,y1,z,h):
 stage['constructions'].append({'id':uid(name),'name':name,'shape':{'kind':'platform','spaceId':space,'outlineMeters':[[dec(x0),dec(y0)],[dec(x1),dec(y0)],[dec(x1),dec(y1)],[dec(x0),dec(y1)]],'baseElevationMeters':dec(z),'heightMeters':dec(h)}})
prism('舞台地面',0,0,10,6,-.15,.15)
prism('独立背墙',0,5.92,10,6,0,6.5)
def rig(name,x,y,z,length,yaw=0):
 stage['constructions'].append({'id':uid(name),'name':name,'shape':{'kind':'rig','rigKind':'truss','spaceId':space,'positionMeters':vec(x,y,z),'yawDegrees':dec(yaw),'lengthMeters':dec(length),'widthMeters':'0.3','heightMeters':'0.3'}})
 return uid(name)
front=rig('前区面光桁架',5,.5,6.3,9.4)
mid=rig('中区效果桁架',5,3,6.3,9.4)
rear=rig('后区逆光桁架',5,5.55,6.3,9.4)
left=rig('左侧桁架',.25,3,6.3,5,90)
right=rig('右侧桁架',9.75,3,6.3,5,90)
roles=[('beam','光束摇头',True),('wash','染色摇头',True),('front','面光摇头',True),('par','全彩帕灯',False)]
for role,label,moving in roles:
 attrs=['dimmer','red','green','blue']+(['pan','tilt'] if moving else [])
 p={'id':uid('profile-'+role),'revision':uid('profile-revision-'+role),'name':label+' · 通用预演档案','manufacturer':'通用','model':label,'mode':'8 通道 / 16 位位置' if moving else '4 通道 RGBD','footprint':8 if moving else 4,'attributes':[{'key':a,'valueType':{'kind':'normalized'},'default':{'kind':'normalized','value':32768 if a in ('pan','tilt') else 0},'mix':'htp' if a=='dimmer' else 'ltp'} for a in attrs],'channels':[{'attribute':a,'encoding':'u8','offsets':[i]} for i,a in enumerate(attrs[:4])]}
 if moving:
  p['channels'] += [{'attribute':'pan','encoding':'u16-be','offsets':[4,5]},{'attribute':'tilt','encoding':'u16-be','offsets':[6,7]}]
  p['positioning']={'kind':'intersectingOrthogonal','pan':{'minDegrees':'-270','maxDegrees':'270','reversed':False},'tilt':{'minDegrees':'-135','maxDegrees':'135','reversed':False}}
 lighting['profiles'].append(p)
records=[]
def fixture(role,x,y,z,location,support=None,rotation=None):
 index=sum(r['role']==role for r in records)+1
 label=next(l for r,l,m in roles if r==role)
 fid=uid(f'{role}-{index}')
 records.append({'id':fid,'role':role,'x':x,'y':y,'z':z,'index':index,'floor':z<1})
 lighting['fixtures'].append({'id':fid,'name':f'{label} {index:02} · {location}','domainId':domain,'profileId':uid('profile-'+role)})
 stage['placements'].append({'fixtureId':fid,'spaceId':space,'positionMeters':vec(x,y,z),'rotationDegreesXYZ':vec(*(rotation or ((180,0,0) if z<1 else (0,0,0))))})
 if support:stage['attachments'].append({'fixtureId':fid,'constructionId':support})
def xs(n):return [.7+i*8.6/(n-1) for i in range(n)]
for x in xs(12):fixture('beam',x,5.55,5.85,'后区逆光',rear)
for x in xs(6):fixture('beam',x,3,5.85,'中区光束',mid)
for x in xs(6):fixture('beam',x,5.25,.32,'后区地排')
for x in xs(8):fixture('wash',x,3.3,5.85,'中区染色',mid)
for x,s in [(.25,left),(9.75,right)]:
 for y in [1.7,4.3]:fixture('wash',x,y,5.85,'侧区染色',s)
for x in [1.8,3.9,6.1,8.2]:fixture('wash',x,4.55,.32,'地排染色')
for x in xs(8):fixture('front',x,.5,5.85,'前区面光',front)
for x in xs(12):fixture('par',x,5.55,.22,'背墙染色',rotation=(170,0,0))
for x in xs(12):fixture('par',x,.35,.22,'前沿轮廓',rotation=(145,0,0))
for x,s in [(.25,left),(9.75,right)]:
 for y in [1.05,2.35,3.65,4.95]:fixture('par',x,y,5.7,'侧区顶染',s,rotation=(0,20 if x<5 else -20,0))
address=1
for r in records:
 lighting['patches'].append({'fixtureId':r['id'],'domainId':domain,'universe':1,'address':address});address+=4 if r['role']=='par' else 8
assert address==513 and len(records)==80
for role,label,moving in roles:
 lighting['groups'].append({'id':uid('group-'+role),'name':f'{label} · {sum(r["role"]==role for r in records)} 台','fixtureIds':[r['id'] for r in records if r['role']==role]})
for name,predicate in [('全部灯具',lambda r:True),('全部摇头灯',lambda r:r['role']!='par'),('地排摇头灯',lambda r:r['floor'] and r['role']!='par')]:
 lighting['groups'].append({'id':uid('group-'+name),'name':name,'fixtureIds':[r['id'] for r in records if predicate(r)]})
def assign(fid,attr,value):return {'operation':'set','target':{'fixtureId':fid,'attribute':attr},'source':{'kind':'literal','value':{'kind':'normalized','value':value}}}
def effects(key):
 out=[]
 palettes={'beam':[(.04,.65,1),(.65,.03,1),(1,.08,.35),(.05,.9,1)],'wash':[(.3,.02,1),(.03,.25,1),(1,.02,.3),(.8,.05,1)],'par':[(.03,.3,1),(.7,.01,.6),(1,.18,.02),(.01,.75,.65)]}
 for role in palettes:
  ids=[r['id'] for r in records if r['role']==role]
  common={'enabled':True,'fixtureIds':ids,'phaseDegrees':0,'reverse':role=='wash','spreadDegrees':360,'dutyPercent':45}
  colors=palettes[role]
  out.append(dict(common,id=uid(key+'color'+role),name={'beam':'光束霓虹流动','wash':'染色紫蓝渐变','par':'帕灯色彩波浪'}[role],periodMs=6400,waveform='keyframes',channels=[{'attribute':a,'keyframes':[{'position':j*2500,'value':norm(c[i]),'transition':'smooth'} for j,c in enumerate(colors)]} for i,a in enumerate(['red','green','blue'])]))
  out.append(dict(common,id=uid(key+'dimmer'+role),name={'beam':'光束错相追逐','wash':'染色呼吸','par':'帕灯滚动追逐'}[role],periodMs=2400 if role!='wash' else 4800,waveform='smooth',channels=[{'attribute':'dimmer','low':norm(.30 if role=='beam' else .12),'high':norm(.95 if role=='beam' else .55)}]))
 return out
families=[('环绕波浪',16,1000),('对称扇形',4,2600),('交叉扫动',4,2200),('中心汇聚',4,2500)]
steps=[];family_steps={}
for family,count,fade in families:
 family_steps[family]=[]
 for n in range(count):
  key=f'{family}-{n}';sid=uid(key);phase=2*math.pi*n/count;assignments=[]
  for r in records:
   fid=r['id'];role=r['role'];x=r['x'];f=(x-5)/4.3
   col=(1,.78,.55) if role=='front' else ((.05,.6,1) if role=='beam' else (.55,.04,1))
   assignments += [assign(fid,'dimmer',norm(.22 if role=='front' else .7))]+[assign(fid,a,norm(c)) for a,c in zip(['red','green','blue'],col)]
   if role=='par':continue
   if role=='front':pan=0;tilt=24
   elif r['floor']:
    pan=25*f+22*math.sin(phase+f);tilt=36+16*math.cos(phase+f)
   else:
    if family=='环绕波浪':pan=180+42*math.sin(phase+f*1.8);tilt=30+16*math.cos(phase+f*1.8)
    elif family=='对称扇形':pan=180+f*(20+50*(.5+.5*math.cos(phase)));tilt=25+13*math.sin(phase)
    elif family=='交叉扫动':pan=180-f*48*math.cos(phase);tilt=32+15*math.sin(phase+(0 if f<0 else math.pi))
    else:
     dx=(5+2.2*math.sin(phase))-x;dy=(1.2+1.0*math.cos(phase))-r['y']
     pan=math.degrees(math.atan2(-dx,dy));pan=pan+360 if pan<0 else pan
     if pan>270:pan-=360
     tilt=math.degrees(math.atan2(math.hypot(dx,dy),r['z']))
    if role=='wash':tilt*=.8
   assignments += [assign(fid,'pan',norm((pan+270)/540)),assign(fid,'tilt',norm((tilt+135)/270))]
  lighting['scenes'].append({'id':sid,'name':f'{family} · {n+1:02}','assignments':assignments,'effects':effects(key)})
  step={'id':uid('step-'+key),'name':f'{family} · {n+1:02}','number':str(len(steps)+1),'sceneId':sid,'actionIds':[],'delay':dur(0),'fade':dur(fade),'advance':{'kind':'after','wait':dur(0)}}
  steps.append(step);family_steps[family].append(step)
def sequence(name,source):
 cloned=copy.deepcopy(source)
 for i,s in enumerate(cloned):s['id']=uid('step-'+name+'-'+str(i));s['number']=str(i+1)
 return {'id':uid('list-'+name),'name':name,'tracking':'isolated','repeat':'loop','release':'profile-defaults','steps':cloned}
lighting['sequences']=[sequence('星河现场 · 全场自动巡演',steps)]+[sequence(name+' · 循环',ss) for name,ss in family_steps.items()]
project={'format':'stagemaster.project','formatVersion':'0.1.0-draft.1','semanticsVersion':'0.1.0-draft.1','project':{'id':uid('project'),'name':'星河现场 · 80灯动态舞台','description':'10 × 6 × 6.5 米；独立背墙与地面，五道桁架，80 台通用灯具。四组运动节目与调光／颜色动态效果。仅供软件预演，光学为当前通用模型；不代表现场吊挂或电力设计。','revisionId':uid('revision-1'),'parentRevisionIds':[]},'requires':[{'key':k,'version':1} for k in ['lighting.basic','stage.layout','stage.spaces','stage.rigging','lighting.effects.basic','lighting.effects.keyframes','lighting.positioning']],'domains':[{'id':domain,'kind':'lighting','name':'灯光'}],'lighting':lighting,'stage':stage}
for k in ['actions','conditions','entryPoints','extensions','resources','rules','syncGroups','timelines']:project[k]=[]
from lighting_layers import apply_layered_looks
apply_layered_looks(project)
DEST.parent.mkdir(parents=True,exist_ok=True)
DEST.write_text(json.dumps(project,ensure_ascii=False,separators=(',',':'))+'\n')
print(DEST);print(f'{len(records)} 台灯具，{len(steps)} 场景，{len(lighting["sequences"])} 列表，{DEST.stat().st_size} 字节')
