// Offline contract checks only; no lighting evaluator or runtime authority.
const assert = (ok, message) => { if (!ok) throw new Error(message); };
const families = new Set(['dimmer','blue,green,red','blue,green,red,white','blue,dimmer,green,red','blue,dimmer,green,red,white']);
export function auditFixtureEmitters(project) {
  for (const profile of project.lighting?.profiles ?? []) {
    const scoped = profile.attributes.filter(a => a.key.startsWith('emitter.'));
    if (!profile.emitters && !scoped.length) continue;
    assert(project.requires.some(c => c.key === 'lighting.fixture-emitters' && c.version === 1), '独立光源缺少能力声明');
    assert(profile.emitters?.length > 0, '独立光源缺少单元定义');
    const owners = new Map();
    for (const unit of profile.emitters) {
      assert(!owners.has(unit.key) && unit.name.trim().length > 0, '光源标识重复或名称无效');
      owners.set(unit.key, []);
    }
    for (const a of scoped) {
      const match = /^emitter\.([a-z][a-z0-9-]{0,31})\.([a-z-]+)$/.exec(a.key);
      assert(match && owners.has(match[1]), '光源属性归属无效');
      const base = match[2];
      assert(a.valueType.kind === 'normalized' && a.default.kind === 'normalized' && a.mix === (base === 'dimmer' ? 'htp' : 'ltp'), '独立光源只支持连续属性及明确混合方式');
      assert(!profile.channels.find(c => c.attribute === a.key)?.functions, '独立光源不能承载功能或自主程序');
      owners.get(match[1]).push(base);
    }
    for (const keys of owners.values()) assert(families.has(keys.sort().join(',')), '光源属性需要完整调光、RGB 或 RGBW 组合');
    assert(!profile.attributes.some(a => ['red','green','blue','white'].includes(a.key)), '根级颜色需要归入光源');
    const master = profile.attributes.find(a => a.key === 'dimmer');
    assert(!master || (master.valueType.kind === 'normalized' && master.mix === 'htp'), '总调光须为线性高值优先');
  }
}
