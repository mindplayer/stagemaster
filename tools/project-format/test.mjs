import test from 'node:test';
import assert from 'node:assert/strict';
import { auditDocuments, auditProject, loadExamples, parseStrict, validateStructure } from './check.mjs';
const examples = loadExamples();
const basic = () => structuredClone(examples.find(d => d.project?.name === '单路灯光示例'));
const room = () => structuredClone(examples.find(d => d.project?.name === '密室声光电联动示例'));
const invalid = (name, make, mutate, pattern) => test(name, () => { const p = make(); mutate(p); assert.throws(() => auditProject(p), pattern); });
const withEffect = () => {
  const p=basic();
  p.requires.push({key:'lighting.effects.basic',version:1});
  p.lighting.scenes[0].effects=[{id:'29999999-0000-4000-8000-000000000001',name:'追逐',enabled:true,fixtureIds:[p.lighting.fixtures[0].id],periodMs:1000,spreadDegrees:360,phaseDegrees:0,reverse:false,waveform:'pulse',dutyPercent:25,channels:[{attribute:'dimmer',low:0,high:65535}]}];
  return p;
};
test('动态效果结构与引用可独立审核',()=>assert.doesNotThrow(()=>auditProject(withEffect())));
invalid('效果必须声明能力',withEffect,p=>{p.requires=p.requires.filter(c=>c.key!=='lighting.effects.basic');},/能力声明/);
invalid('效果引用灯具必须存在',withEffect,p=>{p.lighting.scenes[0].effects[0].fixtureIds=[p.project.id];},/种类错误/);
invalid('不能同时启用争用同属性的效果',withEffect,p=>{const copy=structuredClone(p.lighting.scenes[0].effects[0]);copy.id='29999999-0000-4000-8000-000000000002';p.lighting.scenes[0].effects.push(copy);},/效果目标/);

test('全部设计文档通过结构、对象及跨文件引用检查', () => assert.equal(auditDocuments(examples), 5));
test('严格 JSON 保留字符串精度及合法中文', () => assert.deepEqual(parseStrict('{"名称":"音乐😀","ticks":"9223372036854775807"}'), { 名称:'音乐😀',ticks:'9223372036854775807' }));
for (const [name, text, pattern] of [
 ['重复键','{"id":1,"id":2}',/重复键/],
 ['转义后的重复键','{"id":1,"\\u0069d":2}',/重复键/],
 ['注释','{/*备注*/"a":1}',/注释/],
 ['尾逗号','{"a":1,}',/语法/],
 ['非有限数值','{"a":1e999}',/非有限/],
 ['不精确的大整数','{"a":9007199254740993}',/不精确/],
 ['过深嵌套','['.repeat(65)+'0'+']'.repeat(65),/深度/],
 ['孤立代理字符','"\\ud800"',/Unicode/],
 ['BOM','\uFEFF{}',/BOM/],
]) test(`拒绝${name}`, () => assert.throws(() => parseStrict(text), pattern));
invalid('拒绝运行状态混入工程', basic, p => { p.runtime = { playing:true }; }, /结构/);
invalid('拒绝可编辑到期字段充当授权', basic, p => { p.expiresAt = '2099-01-01'; }, /结构/);
invalid('拒绝原始脚本动作', room, p => { p.actions[0].kind = 'script.execute'; p.actions[0].script = 'move()'; }, /结构/);
invalid('拒绝重复对象身份', basic, p => { p.lighting.groups[0].id = p.lighting.fixtures[0].id; }, /重复对象/);
invalid('拒绝将场景引用指向预设', basic, p => { p.lighting.sequences[0].steps[0].sceneId = p.lighting.presets[0].id; }, /种类错误/);
invalid('显示编号的不同拼写不能规避重复检查', basic, p => { p.lighting.sequences[0].steps[1].number = '1.0'; }, /显示编号/);
invalid('拒绝缺失模块能力声明', room, p => { p.requires = p.requires.filter(c => c.key !== 'motion.external'); }, /能力声明/);
invalid('拒绝负时长', basic, p => { p.lighting.sequences[0].steps[0].fade.ticks = '-1'; }, /结构/);
invalid('拒绝零时基', basic, p => { p.lighting.sequences[0].steps[0].fade.ticksPerSecond = '0'; }, /结构/);
invalid('拒绝时间整数溢出', basic, p => { p.lighting.sequences[0].steps[0].fade.ticks = '9223372036854775808'; }, /i64/);
invalid('拒绝超出时基限制', basic, p => { p.lighting.sequences[0].steps[0].fade.ticksPerSecond = '1000000001'; }, /时基/);
invalid('拒绝把毫米轴变成角度动作', room, p => { const a=p.actions.find(a=>a.kind==='motion.move'); a.request={kind:'angular',position:{value:'200',unit:'degree'},speed:{value:'20',unit:'degree/s'}}; }, /单位不符/);
invalid('拒绝运动请求越过声明行程', room, p => { p.actions.find(a=>a.kind==='motion.move').request.position.value='301'; }, /行程/);
invalid('拒绝缺失运动就绪条件', room, p => { p.actions.find(a=>a.kind==='motion.move').requiresConditions=[]; }, /就绪/);
invalid('机构超时不能只提示并继续推进', room, p => { p.actions.find(a=>a.kind==='motion.move').policy.onTimeout='notify'; }, /超时/);
invalid('动作完成期限不能短于确认期限', room, p => { p.actions[0].policy.completionTimeout.ticks='1'; }, /期限/);
invalid('拒绝错误位置反馈来源', room, p => { p.io.signals[0].sourceId=p.io.devices[0].id; }, /反馈/);
invalid('未知反馈必须传播，不能在取反前强制为假', room, p => { p.conditions[0].onUnknown='false'; }, /结构/);
invalid('拒绝条件循环', room, p => { p.conditions[0].expression={kind:'not',conditionId:p.conditions[0].id}; }, /循环/);
invalid('拒绝条件放宽信号有效期', room, p => { p.conditions[0].expression.maxAge.ticks='101'; }, /有效期/);
invalid('拒绝给布尔输出设置调光数值', room, p => { p.actions.find(a=>a.kind==='io.set').value={kind:'normalized',value:100}; }, /类型不符/);
invalid('拒绝超出单路通道边界', basic, p => { p.lighting.patches[0].address=511; }, /512/);
invalid('拒绝两灯配适冲突', basic, p => { const f=structuredClone(p.lighting.fixtures[0]); f.id='10000000-0000-4000-8000-000000000001'; p.lighting.fixtures.push(f); p.lighting.patches.push({...p.lighting.patches[0],fixtureId:f.id}); }, /通道冲突/);
invalid('拒绝错误的 16 位编码宽度', basic, p => { p.lighting.profiles[0].channels[0].encoding='u16-be'; }, /编码宽度/);
invalid('跨时基精确核对素材长度', room, p => { p.timelines[0].tracks[1].items[0].sourceIn.ticks='1'; }, /素材长度/);
invalid('媒体外部播放入点不可为负', room, p => { p.actions[0].position.ticks='-1'; }, /媒体定位/);
invalid('拒绝轨道类型错配', room, p => { p.timelines[0].tracks[1].kind='events'; }, /轨道/);
invalid('结束点不能携带可能在循环边界重放的动作', room, p => { p.timelines[0].tracks[3].items[0].at.ticks='30000'; }, /结束点/);
invalid('拒绝界面推子反复触发机构动作', room, p => { const c=p.surfaces.pages[0].controls[2]; c.kind='fader'; c.behavior='absolute-pickup'; }, /瞬时按钮/);
invalid('拒绝入口引用未知互动规则', room, p => { p.entryPoints[0].ruleIds=[p.actions[0].id]; }, /种类错误/);
invalid('拒绝目录穿越的资源路径', room, p => { p.resources[0].locator={kind:'package',path:'../secret'}; }, /结构/);
invalid('拒绝场景节点父级循环', room, p => { p.stage.nodes[0].parentId=p.stage.nodes[1].id; }, /循环/);
test('显示名称不承担身份，改名不破坏引用', () => { const p=room(); p.lighting.fixtures[0].name='新名称'; auditProject(p); });
test('精确有理数时基可以等价表达时长', () => { const p=room(); p.timelines[0].tracks[1].items[0].duration={ticks:'1440000',ticksPerSecond:'48000'}; auditProject(p); });
test('拒绝现场绑定的过期工程修订', () => { const docs=structuredClone(examples); docs.find(d=>d.binding?.project).binding.project.revisionId='10000000-0000-4000-8000-000000000001'; assert.throws(()=>auditDocuments(docs),/修订不匹配/); });
test('拒绝部署遗漏执行域', () => { const docs=structuredClone(examples); docs.find(d=>d.format==='stagemaster.deployment-manifest').target.domainIds=['10000000-0000-4000-8000-000000000001']; assert.throws(()=>auditDocuments(docs),/引用不存在/); });
test('拒绝在现场绑定内嵌密码', () => { const b=structuredClone(examples.find(d=>d.format==='stagemaster.site-binding')); b.routes[0].password='example'; assert.throws(()=>validateStructure(b),/结构/); });
test('未知扩展可保留，但结构合格并不表示支持其执行', () => { const p=basic(); p.extensions=[{namespace:'com.example.future',version:1,role:'required-semantic',references:[],payload:{newFeature:true}}]; auditProject(p); /* Rust 编译器必须另行拒绝未知必需语义。 */ });

test('空白编辑工程可以保存，但不生成虚假的执行入口', () => {
  const p = basic();
  for (const key of ['fixtures','groups','presets','scenes','sequences','patches']) p.lighting[key] = [];
  p.entryPoints = [];
  assert.doesNotThrow(() => auditProject(p));
});

const spaces = () => {
  const p = basic(), id = '30000000-0000-4000-8000-000000000001';
  p.requires.push({key:'stage.layout',version:1},{key:'stage.spaces',version:1});
  p.stage = {
    nodes:[],
    spaces:[{id,name:'主厅',outlineMeters:[['0','0'],['8','0'],['8','4'],['0','4']],floorElevationMeters:'0',clearHeightMeters:'5'}],
    constructions:[{id:'30000000-0000-4000-8000-000000000002',name:'围护',shape:{kind:'enclosure',spaceId:id,wallThicknessMeters:'0.2',floorThicknessMeters:'0.1',ceilingThicknessMeters:null}}],
    placements:[{fixtureId:p.lighting.fixtures[0].id,spaceId:id,positionMeters:{x:'1',y:'2',z:'4.5'},rotationDegreesXYZ:{x:'0',y:'15',z:'0'}}],
  };
  return p;
};
test('空间扩展的结构与跨领域引用可核对', () => assert.doesNotThrow(() => auditProject(spaces())));
invalid('空间扩展不可省略新能力声明',spaces,p => {p.requires=p.requires.filter(c=>c.key!=='stage.spaces');},/能力声明/);
invalid('灯位不可引用场景当作灯具',spaces,p => {p.stage.placements[0].fixtureId=p.lighting.scenes[0].id;},/种类错误/);
invalid('拒绝一灯重复布置',spaces,p => {p.stage.placements.push(structuredClone(p.stage.placements[0]));},/灯具布置/);
invalid('空间不能与灯具重复身份',spaces,p => {p.stage.spaces[0].id=p.lighting.fixtures[0].id;},/重复对象身份/);
invalid('灯位归属不可引用已删除空间',spaces,p => {p.stage.spaces=[];},/引用不存在/);
invalid('围护需要有界空间净高',spaces,p => {p.stage.spaces[0].clearHeightMeters=null;},/净高/);
invalid('安装旋转与位置都有上限',spaces,p => {p.stage.placements[0].rotationDegreesXYZ.x='3601';},/越界/);
invalid('空间坐标保留十进制字符串规范',spaces,p => {p.stage.placements[0].positionMeters.z='4.50';},/结构/);

const withKeyframes=()=>{
  const p=withEffect();p.requires.push({key:'lighting.effects.keyframes',version:1});
  const e=p.lighting.scenes[0].effects[0];e.waveform='keyframes';
  e.channels=[{attribute:'dimmer',keyframes:[{position:0,value:0,transition:'linear'},{position:5000,value:65535,transition:'hold'}]}];return p;
};
test('关键帧结构和能力声明通过独立校验',()=>assert.doesNotThrow(()=>auditProject(withKeyframes())));
invalid('关键帧不能缺少专用能力声明',withKeyframes,p=>{p.requires=p.requires.filter(c=>c.key!=='lighting.effects.keyframes')},/能力声明/);
invalid('关键帧位置必须严格递增',withKeyframes,p=>{p.lighting.scenes[0].effects[0].channels[0].keyframes[1].position=0},/递增/);
invalid('关键帧不接受被忽略的两端值',withKeyframes,p=>{p.lighting.scenes[0].effects[0].channels[0].low=0},/结构/);
invalid('关键帧与基本变化方式不能混用',withKeyframes,p=>{p.lighting.scenes[0].effects[0].waveform='smooth'},/变化方式/);

const rigging = () => {
  const p = spaces();
  p.requires.push({key:'stage.rigging',version:1});
  const c={id:'30000000-0000-4000-8000-000000000090',name:'前桁架',shape:{kind:'rig',rigKind:'truss',spaceId:p.stage.spaces[0].id,positionMeters:{x:'4',y:'3',z:'5'},yawDegrees:'0',lengthMeters:'6',widthMeters:'0.3',heightMeters:'0.4'}};
  p.stage.constructions.push(c);p.stage.attachments=[{fixtureId:p.stage.placements[0].fixtureId,constructionId:c.id}];return p;
};
test('支撑体挂接格式有效',()=>auditDocuments([rigging()]));
invalid('挂接不可省略能力声明',rigging,p=>{p.requires=p.requires.filter(c=>c.key!=='stage.rigging');},/能力声明/);
invalid('一台灯不可重复挂接',rigging,p=>{p.stage.attachments.push({...p.stage.attachments[0]});},/重复/);
invalid('挂接不可悬空',rigging,p=>{p.stage.placements=[];},/灯位/);
invalid('挂接空间不可分离',rigging,p=>{p.stage.placements[0].spaceId=null;},/空间必须一致/);

const withPosition = () => {
 const p=basic(),profile=p.lighting.profiles[0];
 p.requires.push({key:'lighting.positioning',version:1});
 profile.positioning={kind:'intersectingOrthogonal',pan:{minDegrees:'-270',maxDegrees:'270',reversed:true},tilt:{minDegrees:'-135',maxDegrees:'135',reversed:false}};
 const width=profile.footprint;
 for(const [i,key] of ['pan','tilt'].entries()) {profile.attributes.push({key,valueType:{kind:'normalized'},default:{kind:'normalized',value:32768},mix:'ltp'});profile.channels.push({attribute:key,encoding:'u16-be',offsets:[width+i*2,width+i*2+1]});}
 profile.footprint+=4;
 let address=1;for(const patch of p.lighting.patches){patch.address=address;address+=p.lighting.profiles.find(pr=>pr.id===p.lighting.fixtures.find(f=>f.id===patch.fixtureId).profileId).footprint;}
 p.lighting.fixtures[0].zeroCorrection={panDegrees:'2.5',tiltDegrees:'-3'};
 return p;
};
test('两轴物理定义、通道映射和手工零偏可独立核验',()=>assert.doesNotThrow(()=>auditProject(withPosition())));
invalid('摇头灯档案必须声明能力',withPosition,p=>{p.requires=p.requires.filter(c=>c.key!=='lighting.positioning');},/位置能力/);
invalid('物理行程不可倒置',withPosition,p=>{p.lighting.profiles[0].positioning.pan.maxDegrees='-280';},/物理范围/);
invalid('两轴不能丢失位置通道',withPosition,p=>{p.lighting.profiles[0].channels=p.lighting.profiles[0].channels.filter(c=>c.attribute!=='pan');},/位置属性/);
invalid('手工轴零偏有界',withPosition,p=>{p.lighting.fixtures[0].zeroCorrection.panDegrees='361';},/零偏/);
