import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import Ajv2020 from 'ajv/dist/2020.js';
import { createScanner, parseTree } from 'jsonc-parser';

// Development-only contract review. This is deliberately not a runtime loader or compiler.
export const root = fileURLToPath(new URL('../../docs/project-format/', import.meta.url));
const version = '0.1.0-draft.1';
const schemaId = name => `urn:stagemaster:schema:${name}:${version}`;
const fail = message => { throw new Error(message); };
const assert = (ok, message) => { if (!ok) fail(message); };

export function parseStrict(text) {
  assert(Buffer.byteLength(text, 'utf8') <= 8 * 1024 * 1024, 'JSON 超过开发期 8 MiB 限制');
  assert(!text.startsWith('\uFEFF'), 'JSON 不接受 BOM');
  const scanner = createScanner(text);
  let depth = 0, count = 0;
  for (let token = scanner.scan(); token !== 17; token = scanner.scan()) {
    assert(++count <= 1_000_000, 'JSON 标记数量超限');
    if (token === 1 || token === 3) assert(++depth <= 64, 'JSON 嵌套深度超限');
    if (token === 2 || token === 4) depth--;
    assert(scanner.getTokenError() === 0, 'JSON 词法错误');
    assert(token !== 12 && token !== 13, 'JSON 不接受注释');
  }
  const errors = [];
  const tree = parseTree(text, errors, { disallowComments: true, allowTrailingComma: false });
  assert(tree && errors.length === 0, 'JSON 语法错误');
  function visit(node, path) {
    if (node.type === 'object') {
      const seen = new Set();
      for (const property of node.children ?? []) {
        const key = property.children[0].value;
        assert(!seen.has(key), `JSON 重复键：${path}/${key}`);
        seen.add(key);
      }
    }
    if (node.type === 'number') assert(Number.isFinite(node.value) && (!Number.isInteger(node.value) || Number.isSafeInteger(node.value)), `JSON 数字不精确或非有限：${path}`);
    if (node.type === 'string') {
      // Reject isolated surrogate code units, including those introduced by escapes.
      assert(!/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/u.test(node.value), `JSON 字符串含无效 Unicode：${path}`);
    }
    (node.children ?? []).forEach((child, i) => visit(child, `${path}/${i}`));
  }
  visit(tree, '');
  return JSON.parse(text);
}

export function readJson(path) {
  // Fatal decoding rejects malformed UTF-8 instead of silently replacing bytes.
  return parseStrict(new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(readFileSync(path)));
}
const ajv = new Ajv2020({ strict: true, allErrors: true, validateFormats: false });
for (const name of ['common', 'project', 'site-binding', 'deployment']) ajv.addSchema(readJson(resolve(root, 'schemas', `${name}.schema.json`)));
const validators = new Map([
  ['stagemaster.project', ajv.getSchema(schemaId('project'))],
  ['stagemaster.site-binding', ajv.getSchema(schemaId('site-binding'))],
  ['stagemaster.deployment-manifest', ajv.getSchema(schemaId('deployment'))],
]);
export function validateStructure(document) {
  const validate = validators.get(document?.format);
  assert(validate, '未知文件格式');
  assert(validate(document), `结构不符合 Schema：${JSON.stringify(validate.errors?.slice(0, 4))}`);
  function bounds(value) {
    if (!value || typeof value !== 'object') return;
    if ('ticks' in value && 'ticksPerSecond' in value) {
      assert(BigInt(value.ticks) >= -(2n ** 63n) && BigInt(value.ticks) < 2n ** 63n, '时间刻度超出 i64 范围');
      assert(BigInt(value.ticksPerSecond) <= 1_000_000_000n, '时基超出 10^9 限制');
    }
    if (value.kind === 'integer' && 'value' in value) assert(BigInt(value.value) >= -(2n ** 63n) && BigInt(value.value) < 2n ** 63n, '整数值超出 i64 范围');
    for (const [key, child] of Object.entries(value)) {
      if (['byteLength','planBytes','runtimeRamBytes'].includes(key)) assert(BigInt(child) < 2n ** 64n, '字节数量超出 u64 范围');
      if (key !== 'payload') bounds(child);
    }
  }
  bounds(document);
}

const durationCompare = (a, b) => BigInt(a.ticks) * BigInt(b.ticksPerSecond) - BigInt(b.ticks) * BigInt(a.ticksPerSecond);
const durationAdd = (a, b) => ({ ticks: String(BigInt(a.ticks) * BigInt(b.ticksPerSecond) + BigInt(b.ticks) * BigInt(a.ticksPerSecond)), ticksPerSecond: String(BigInt(a.ticksPerSecond) * BigInt(b.ticksPerSecond)) });
const decimalCompare = (a, b) => {
  const split = value => { const [whole, fraction = ''] = value.replace('-', '').split('.'); return [BigInt(whole + fraction) * (value.startsWith('-') ? -1n : 1n), fraction.length]; };
  const [x, sx] = split(a), [y, sy] = split(b);
  return x * 10n ** BigInt(sy) - y * 10n ** BigInt(sx);
};
const matches = (type, value) => type.kind === value.kind && (type.kind !== 'scalar' || type.unit === value.unit) && (type.kind !== 'choice' || type.options.includes(value.value));
function unique(values, label) { assert(new Set(values).size === values.length, `重复${label}`); }
function acyclic(items, edges, label) {
  const remaining = new Map(), dependents = new Map();
  for (const item of items) {
    const targets = edges(item); remaining.set(item.id, targets.length);
    for (const target of targets) {
      if (!dependents.has(target)) dependents.set(target, []);
      dependents.get(target).push(item.id);
    }
  }
  const ready = [...remaining].filter(([, count]) => count === 0).map(([id]) => id);
  let count = 0;
  while (ready.length) {
    const id = ready.pop(); count++;
    for (const dependent of dependents.get(id) ?? []) {
      const next = remaining.get(dependent) - 1; remaining.set(dependent, next);
      if (next === 0) ready.push(dependent);
    }
  }
  assert(count === items.length, `${label}循环引用`);
}

export function auditProject(p) {
  validateStructure(p);
  const objects = new Map();
  const add = (kind, values = []) => {
    for (const value of values) {
      assert(!objects.has(value.id), `重复对象身份：${value.id}`);
      objects.set(value.id, { kind, value });
    }
  };
  const get = (id, kinds) => {
    const entry = objects.get(id);
    assert(entry && kinds.split('|').includes(entry.kind), `引用不存在或种类错误：${id}，应为 ${kinds}`);
    return entry.value;
  };
  add('project', [p.project]); add('resource', p.resources); add('domain', p.domains);
  const kinds = { profiles:'profile',fixtures:'fixture',groups:'group',presets:'preset',scenes:'scene',sequences:'sequence',systems:'media-system',objects:'media-object',controllers:'controller',axes:'axis',devices:'io-device',signals:'signal',outputs:'output',sources:'monitor-source',views:'monitor-view',pages:'page',nodes:'stage-node',spaces:'stage-space',constructions:'stage-construction' };
  for (const module of ['lighting','media','motion','io','monitoring','surfaces','stage']) for (const [key, values] of Object.entries(p[module] ?? {})) if (kinds[key]) add(kinds[key], values);
  for (const [key, kind] of Object.entries({ syncGroups:'sync-group',actions:'action',conditions:'condition',rules:'rule',timelines:'timeline',entryPoints:'entry-point' })) add(kind, p[key]);
  for (const seq of p.lighting?.sequences ?? []) add('step', seq.steps);
  for (const scene of p.lighting?.scenes ?? []) add('effect', scene.effects);
  for (const t of p.timelines) { add('track', t.tracks); for (const track of t.tracks) add('item', track.items); }
  for (const page of p.surfaces?.pages ?? []) add('control', page.controls);
  unique(p.requires.map(x => x.key), '能力键');
  const declared = new Set(p.requires.map(x => `${x.key}@${x.version}`));
  for (const [module, capability] of Object.entries({ lighting:'lighting.basic',media:'media.external',motion:'motion.external',io:'io.logic',stage:'stage.layout',monitoring:'monitoring',surfaces:'surface.mapping' })) if (p[module]) assert(declared.has(`${capability}@1`), `缺少模块能力声明：${capability}`);
  if (p.stage && ['spaces','constructions','placements'].some(key => key in p.stage)) assert(declared.has('stage.spaces@1'), '缺少模块能力声明：stage.spaces');
  if (p.timelines.length) assert(declared.has('timeline.basic@1'), '缺少时间线能力声明');
  if (p.rules.length) assert(declared.has('automation.rules@1'), '缺少联动能力声明');
  const attr = target => {
    const fixture = get(target.fixtureId, 'fixture');
    const attribute = get(fixture.profileId, 'profile').attributes.find(a => a.key === target.attribute);
    assert(attribute, `未知灯具属性：${target.attribute}`); return attribute;
  };
  const domain = (id, kind) => assert(get(id, 'domain').kind === kind, `执行域种类不符：${id}`);
  for (const profile of p.lighting?.profiles ?? []) {
    if (profile.sourceResourceId) get(profile.sourceResourceId, 'resource');
    unique(profile.attributes.map(a => a.key), '档案属性');
    const offsets = [];
    for (const a of profile.attributes) assert(matches(a.valueType, a.default), '档案默认值类型不符');
    for (const c of profile.channels) {
      assert(profile.attributes.some(a => a.key === c.attribute && a.valueType.kind === 'normalized'), '本草案 DMX 映射仅支持归一化属性');
      assert(c.offsets.length === (c.encoding === 'u8' ? 1 : 2), '通道编码宽度错误');
      assert(c.offsets.every(x => x < profile.footprint), '档案通道越界'); offsets.push(...c.offsets);
    }
    unique(offsets, '档案通道偏移'); unique(profile.channels.map(c => c.attribute), '属性通道映射');
  }
  for (const f of p.lighting?.fixtures ?? []) { get(f.profileId, 'profile'); domain(f.domainId, 'lighting'); }
  for (const g of p.lighting?.groups ?? []) g.fixtureIds.forEach(id => get(id, 'fixture'));
  for (const preset of p.lighting?.presets ?? []) {
    unique(preset.values.map(a => `${a.target.fixtureId}/${a.target.attribute}`), '预设属性');
    for (const a of preset.values) assert(matches(attr(a.target).valueType, a.value), '预设值类型不符');
  }
  for (const scene of p.lighting?.scenes ?? []) {
    const effects = scene.effects ?? [], activeTargets = [];
    if (effects.length) assert(declared.has('lighting.effects.basic@1'), '缺少动态效果能力声明');
    for (const effect of effects) {
      unique(effect.channels.map(c => c.attribute), '效果属性');
      const keyed = effect.waveform === 'keyframes';
      if (keyed) assert(declared.has('lighting.effects.keyframes@1'), '缺少关键帧效果能力声明');
      let timing;
      for (const channel of effect.channels) {
        assert(keyed === Array.isArray(channel.keyframes), '变化方式与关键帧不一致');
        if (keyed) {
          const frames = channel.keyframes;
          assert(frames[0].position === 0 && frames.every((f,i) => i === 0 || f.position > frames[i-1].position), '关键帧须从零开始递增');
          const current = JSON.stringify(frames.map(f => [f.position, f.transition]));
          assert(timing === undefined || timing === current, '关键帧位置与过渡方式须一致');
          timing = current;
        }
      }
      for (const fixtureId of effect.fixtureIds) for (const channel of effect.channels) {
        assert(attr({fixtureId, attribute:channel.attribute}).valueType.kind === 'normalized', '效果仅支持归一化属性');
        if (effect.enabled) activeTargets.push(`${fixtureId}/${channel.attribute}`);
      }
    }
    unique(activeTargets, '已启用效果目标');
    unique(scene.assignments.map(a => `${a.target.fixtureId}/${a.target.attribute}`), '场景属性');
    for (const a of scene.assignments) {
      const attribute = attr(a.target);
      if (a.operation === 'release') continue;
      if (a.source.kind === 'literal') assert(matches(attribute.valueType, a.source.value), '场景值类型不符');
      else assert(get(a.source.presetId, 'preset').values.some(v => v.target.fixtureId === a.target.fixtureId && v.target.attribute === a.target.attribute), '预设不包含被引用属性');
    }
  }
  for (const seq of p.lighting?.sequences ?? []) {
    unique(seq.steps.map(s => Number(s.number)), '场景显示编号');
    for (const step of seq.steps) { get(step.sceneId, 'scene'); step.actionIds.forEach(id => get(id, 'action')); }
  }
  const occupied = new Set(), patched = new Set();
  for (const patch of p.lighting?.patches ?? []) {
    const fixture = get(patch.fixtureId, 'fixture');
    assert(patch.domainId === fixture.domainId, '配适执行域不一致');
    assert(!patched.has(fixture.id), '灯具重复配适'); patched.add(fixture.id);
    const end = patch.address + get(fixture.profileId, 'profile').footprint - 1;
    assert(end <= 512, '配适超出 512 通道');
    for (let channel = patch.address; channel <= end; channel++) {
      const key = `${patch.domainId}/${patch.universe}/${channel}`;
      assert(!occupied.has(key), '配适通道冲突'); occupied.add(key);
    }
  }
  for (const system of p.media?.systems ?? []) domain(system.domainId, 'external-media');
  for (const object of p.media?.objects ?? []) {
    get(object.systemId, 'media-system');
    for (const key of ['resourceId','referenceResourceId']) if (object[key]) get(object[key], 'resource');
  }
  for (const device of p.io?.devices ?? []) domain(device.domainId, 'io');
  for (const signal of p.io?.signals ?? []) get(signal.sourceId, 'controller|media-system|io-device');
  for (const output of p.io?.outputs ?? []) { get(output.deviceId, 'io-device'); assert(matches(output.valueType, output.idleValue), '输出空闲值类型不符'); }
  for (const c of p.conditions) {
    const e = c.expression;
    if (e.kind === 'signal') {
      const signal = get(e.signalId, 'signal');
      assert(matches(signal.valueType, e.value), '条件值类型不符');
      assert(e.operator === 'equals' || ['normalized','scalar','integer'].includes(signal.valueType.kind), '条件比较运算不支持该类型');
      assert(durationCompare(e.maxAge, signal.freshness) <= 0n, '条件不得放宽反馈有效期');
    } else if (e.kind === 'not') get(e.conditionId, 'condition');
    else e.conditionIds.forEach(id => get(id, 'condition'));
  }
  acyclic(p.conditions, c => c.expression.kind === 'signal' ? [] : c.expression.kind === 'not' ? [c.expression.conditionId] : c.expression.conditionIds, '条件');
  for (const c of p.motion?.controllers ?? []) {
    domain(c.domainId, 'motion');
    c.requiredFeedback.forEach(id => assert(get(id, 'signal').sourceId === c.id, '机构反馈来自错误控制器'));
  }
  for (const axis of p.motion?.axes ?? []) {
    const c = get(axis.controllerId, 'controller'), position = get(axis.positionSignalId, 'signal'); get(axis.readyConditionId, 'condition');
    assert(position.sourceId === c.id && position.meaning === 'position' && position.valueType.kind === 'scalar' && position.valueType.unit === axis.unit && c.requiredFeedback.includes(position.id), '机构位置反馈类型或来源错误');
    assert(decimalCompare(axis.requestedTravel.min, axis.requestedTravel.max) < 0n, '机构请求行程范围错误');
  }
  for (const a of p.actions) {
    a.requiresConditions.forEach(id => get(id, 'condition'));
    assert(BigInt(a.policy.ackTimeout.ticks) > 0n && durationCompare(a.policy.completionTimeout, a.policy.ackTimeout) >= 0n, '动作确认／完成期限错误');
    if (a.kind === 'lighting.apply-scene') get(a.sceneId, 'scene');
    else if (a.kind.startsWith('media.') || a.kind === 'external.recall') {
      get(a.objectId, 'media-object');
      if (a.position) assert(BigInt(a.position.ticks) >= 0n, '媒体定位不可为负');
    }
    else if (a.kind === 'motion.move') {
      const axis = get(a.axisId, 'axis');
      assert(a.policy.onTimeout === 'block', '机构移动超时必须阻止后续推进');
      assert(a.requiresConditions.includes(axis.readyConditionId), '机构移动缺少就绪条件');
      assert(a.request.position.unit === axis.unit, '机构动作单位不符');
      assert(decimalCompare(a.request.position.value, axis.requestedTravel.min) >= 0n && decimalCompare(a.request.position.value, axis.requestedTravel.max) <= 0n, '机构请求超出声明行程');
      assert(decimalCompare(a.request.speed.value, '0') > 0n && decimalCompare(a.completionTolerance, '0') > 0n, '机构速度或容差无效');
    } else if (a.kind === 'motion.stop') get(a.controllerId, 'controller');
    else if (a.kind === 'io.set') assert(matches(get(a.outputId, 'output').valueType, a.value), '输出动作值类型不符');
  }
  for (const rule of p.rules) { get(rule.conditionId, 'condition'); rule.actionIds.forEach(id => get(id, 'action')); }
  for (const t of p.timelines) {
    get(t.syncGroupId, 'sync-group'); t.endActionIds.forEach(id => get(id, 'action'));
    for (const track of t.tracks) for (const item of track.items) {
      const allowed = { lighting:['lighting.scene-clip'], 'audio-reference':['media.reference-clip'], 'video-reference':['media.reference-clip'], events:['action-event'], markers:['marker'] };
      assert(allowed[track.kind].includes(item.kind), '轨道与片段类型不符');
      const end = item.start ? durationAdd(item.start, item.duration) : item.at;
      assert(durationCompare(end, t.duration) <= 0n, '时间线内容超出结束点');
      if (item.kind === 'lighting.scene-clip') { get(item.sceneId, 'scene'); assert(durationCompare(item.fadeIn, item.duration) <= 0n, '渐变长于片段'); }
      if (item.kind === 'media.reference-clip') {
        const resource = get(item.resourceId, 'resource');
        assert(resource.kind === (track.kind === 'audio-reference' ? 'audio' : 'video'), '媒体轨道资源类型错误');
        assert(BigInt(item.sourceIn.ticks) >= 0n, '媒体入点不可为负');
        if (resource.media) assert(durationCompare(durationAdd(item.sourceIn, item.duration), resource.media.duration) <= 0n, '媒体片段超出素材长度');
      }
      if (item.kind === 'action-event') { get(item.actionId, 'action'); assert(durationCompare(item.at, t.duration) < 0n, '动作事件不得位于时间线结束点'); }
    }
  }
  for (const entry of p.entryPoints) {
    get(entry.target.id, entry.target.kind); entry.domainIds.forEach(id => get(id, 'domain')); entry.ruleIds.forEach(id => get(id, 'rule')); entry.shutdown.actionIds.forEach(id => get(id, 'action'));
  }
  for (const node of p.stage?.nodes ?? []) {
    if (node.parentId) get(node.parentId, 'stage-node');
    if (node.deviceId) get(node.deviceId, 'fixture|media-system|controller|axis|io-device');
    if (node.resourceId) get(node.resourceId, 'resource');
    assert(Object.values(node.transform.scale).every(x => decimalCompare(x, '0') > 0n), '场景缩放必须为正');
  }
  acyclic(p.stage?.nodes ?? [], n => n.parentId ? [n.parentId] : [], '场景父节点');
  // Geometric validity/triangulation belongs to Rust + geo; this offline tool checks structure,
  // bounds and references only and must not be used as the product's geometric acceptance gate.
  const bounded = (value, min, max) => assert(decimalCompare(value, String(min)) >= 0n && decimalCompare(value, String(max)) <= 0n, '空间数值越界');
  const outlineBounds = points => points.flat().forEach(v => bounded(v, -100000, 100000));
  for (const space of p.stage?.spaces ?? []) {
    outlineBounds(space.outlineMeters); bounded(space.floorElevationMeters, -10000, 10000);
    if (space.clearHeightMeters !== null) bounded(space.clearHeightMeters, 0.1, 1000);
  }
  const enclosures = [];
  for (const construction of p.stage?.constructions ?? []) {
    const s = construction.shape;
    const space = s.spaceId === null ? null : get(s.spaceId, 'stage-space');
    if (s.kind === 'enclosure') {
      enclosures.push(s.spaceId); assert(space.clearHeightMeters !== null, '围护需要空间净高');
      bounded(s.wallThicknessMeters, 0.001, 10); bounded(s.floorThicknessMeters, 0.001, 10);
      if (s.ceilingThicknessMeters !== null) bounded(s.ceilingThicknessMeters, 0.001, 10);
    } else if (s.kind === 'rig') {
      Object.values(s.positionMeters).forEach(v => bounded(v, -100000, 100000));
      bounded(s.yawDegrees, -3600, 3600); bounded(s.lengthMeters, 0.1, 1000); bounded(s.widthMeters, 0.02, 10); bounded(s.heightMeters, 0.02, 10);
    } else {
      outlineBounds(s.outlineMeters); bounded(s.baseElevationMeters, -10000, 10000); bounded(s.heightMeters, 0.001, 1000);
    }
  }
  if ((p.stage?.attachments?.length ?? 0) || p.stage?.constructions?.some(c => c.shape.kind === 'rig')) assert(declared.has('stage.rigging@1'), '缺少模块能力声明：stage.rigging');
  unique((p.stage?.attachments ?? []).map(a => a.fixtureId), '灯具挂接');
  for (const a of p.stage?.attachments ?? []) {
    const c = get(a.constructionId, 'stage-construction');
    const placement = p.stage?.placements?.find(p => p.fixtureId === a.fixtureId);
    assert(c.shape.kind === 'rig' && placement, '挂接需要支撑体和灯位');
    assert(c.shape.spaceId === placement.spaceId, '挂接空间必须一致');
  }
  unique(enclosures, '空间围护');
  unique((p.stage?.placements ?? []).map(p => p.fixtureId), '灯具布置');
  for (const placement of p.stage?.placements ?? []) {
    get(placement.fixtureId, 'fixture');
    if (placement.spaceId !== null) get(placement.spaceId, 'stage-space');
    Object.values(placement.positionMeters).forEach(v => bounded(v, -100000, 100000));
    Object.values(placement.rotationDegreesXYZ).forEach(v => bounded(v, -3600, 3600));
  }

  for (const source of p.monitoring?.sources ?? []) { get(source.systemId, 'media-system'); if (source.referenceResourceId) get(source.referenceResourceId, 'resource'); }
  for (const view of p.monitoring?.views ?? []) get(view.sourceId, 'monitor-source');
  for (const page of p.surfaces?.pages ?? []) for (const control of page.controls) {
    if (control.target.kind === 'lighting-attribute') attr(control.target.target);
    else { get(control.target.actionId ?? control.target.entryPointId, control.target.kind); assert(control.kind === 'button' && control.behavior === 'momentary', '离散动作只能绑定瞬时按钮'); }
  }
  for (const extension of p.extensions) for (const reference of extension.references) get(reference.id, reference.kind);
  return { get, objects };
}

export function auditDocuments(documents) {
  documents.forEach(validateStructure);
  const projects = new Map(), bindings = new Map();
  for (const p of documents.filter(d => d.format === 'stagemaster.project')) {
    const key = `${p.project.id}/${p.project.revisionId}`; assert(!projects.has(key), '重复工程修订');
    projects.set(key, { p, ...auditProject(p) });
  }
  const project = ref => {
    const result = projects.get(`${ref.projectId}/${ref.revisionId}`);
    assert(result, '跨文件工程或修订不匹配'); return result;
  };
  for (const b of documents.filter(d => d.format === 'stagemaster.site-binding')) {
    const context = project(b.binding.project), { get } = context;
    const key = `${b.binding.id}/${b.binding.revisionId}`; assert(!bindings.has(key), '重复现场绑定修订'); bindings.set(key, { b, context });
    unique(b.routes.map(r => r.kind === 'dmx' ? `${r.domainId}/${r.universe}` : r.objectId), '现场路由');
    for (const r of b.routes) {
      if (r.kind === 'dmx') assert(get(r.domainId, 'domain').kind === 'lighting', 'DMX 绑定执行域错误');
      else {
        const object = get(r.objectId, 'media-system|controller|io-device');
        assert(object.adapterContract.key === r.adapterContract.key && object.adapterContract.version === r.adapterContract.version, '适配器契约不匹配');
        if (object.externalProject) assert(object.externalProject.key === r.expectedRemoteProject.key && object.externalProject.revision === r.expectedRemoteProject.revision, '外部工程修订不匹配');
      }
    }
    for (const r of b.monitorRoutes) get(r.sourceId, 'monitor-source');
    for (const r of b.surfaceRoutes) {
      const page = get(r.pageId, 'page');
      unique(r.controlMap.map(c => c.logicalControlId), '控制面映射'); unique(r.controlMap.map(c => c.physicalControlKey), '物理控制器映射');
      for (const c of r.controlMap) assert(page.controls.some(control => control.id === c.logicalControlId), '控制面引用了其他页面的控件');
    }
    for (const c of b.commissioning) get(c.controllerId, 'controller');
  }
  for (const d of documents.filter(d => d.format === 'stagemaster.deployment-manifest')) {
    const { p, get } = project(d.source), binding = bindings.get(`${d.binding.id}/${d.binding.revisionId}`);
    assert(binding && binding.context.p === p, '部署的现场绑定或工程不匹配');
    d.target.domainIds.forEach(id => get(id, 'domain'));
    for (const id of d.entryPointIds) assert(get(id, 'entry-point').domainIds.every(domain => d.target.domainIds.includes(domain)), '部署缺失入口要求的执行域');
    assert(BigInt(d.plan.byteLength) <= BigInt(d.budgets.planBytes), '部署计划超出声明字节预算');
  }
  return documents.length;
}
export function loadExamples() {
  return readdirSync(resolve(root, 'examples')).filter(n => n.endsWith('.json')).sort().map(name => readJson(resolve(root, 'examples', name)));
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const docs = process.argv.length > 2 ? process.argv.slice(2).map(path => readJson(resolve(path))) : loadExamples();
    console.log(`通过：4 份 Schema，${auditDocuments(docs)} 份设计文档的结构与引用检查。`);
    console.log('未验证媒体文件、编译计划、签名授权、设备容量或现场保护；样例不可直接部署。');
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
