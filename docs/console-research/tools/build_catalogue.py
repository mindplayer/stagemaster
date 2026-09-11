"""Build traceable research indexes from module tables; no network or dependencies.

Retrieval status is never promoted to reviewed/tested. Evidence remains descriptive.
Run after editing module documents: python3 docs/console-research/tools/build_catalogue.py
"""
from collections import Counter, defaultdict
import json
from pathlib import Path
import re
from urllib.parse import urldefrag

ROOT = Path(__file__).resolve().parents[1]
DATE = '2026-09-10'
LINK = re.compile(r'\[[^\]]*\]\((https?://[^\s)]+)\)')

MA_MAP = {
    'grandMA3 User Manual': 'M22', 'New in the Manual': 'M21', 'About the Help': 'M20',
    'Device Overview': 'M01 M20', 'System Overview': 'M01 M16 M18',
    'First Steps': 'M02 M03 M04 M06 M08 M20', 'grandMA3 onPC': 'M01 M20 M21',
    'Show File Handling': 'M03', 'Workspace': 'M02', 'Command Syntax and Keywords': 'M15',
    'Windows, Views, and Menus': 'M02', 'Networking': 'M16 M17 M18',
    'DMX In and Out': 'M16', 'Single User and Multi User Systems': 'M02 M18',
    'Patch and Fixture Setup': 'M04 M14', 'Operate Fixtures': 'M05 M06 M11',
    'Label Objects': 'M02', 'Notes': 'M02 M19', 'Scribbles': 'M02',
    'Appearances': 'M02', 'Images': 'M02 M12', 'Supported File Formats': 'M03 M12',
    'Screenshots': 'M02 M20', 'Meshes': 'M14', 'Videos': 'M12', 'Gobos': 'M04 M06',
    'Symbols': 'M02 M05', 'Groups': 'M05 M09', 'Presets': 'M07',
    'Worlds and Filters': 'M06 M18', 'MAtricks and Shuffle': 'M05 M10 M11',
    'Cues and Sequences': 'M08 M09', 'Executors': 'M09 M19', 'Masters': 'M09 M19',
    'Recipes': 'M11', 'Phasers': 'M10', 'Shapes': 'M10 M11',
    'Generator - Random': 'M10', 'Bitmap': 'M12', 'XYZ': 'M14 M17', 'Tags': 'M02 M19',
    'Macros': 'M15', 'Agenda': 'M15', 'Timers': 'M13 M15', 'Preview': 'M14',
    'Timecode Show': 'M13', 'Layouts': 'M02 M05', 'Plugins': 'M15', 'Quickeys': 'M02 M15',
    'Data Pools': 'M03 M18', 'System': 'M20', 'Remote In and Out': 'M17', 'Sound': 'M13',
    'RDM (Remote Device Management)': 'M04 M16', 'Local Settings': 'M02 M20',
    'Update the Software': 'M20 M21', 'Fixture Types': 'M04', 'File Management': 'M03 M20',
    'Show Creator': 'M03 M07 M11', 'Control other MA Devices': 'M16 M17 M20',
    'Troubleshooting': 'M20', 'Glossary': 'M22', 'grandMA3 List of Trademarks': 'M22',
}
TITAN_MAP = {
    'Introduction': 'M01 M22', 'about-the-consoles': 'M01 M20', 'alpha-index': 'M22',
    'capture-visualiser': 'M14', 'chases': 'M08 M09', 'controlling-fixtures': 'M05 M06',
    'cue-lists': 'M08 M09', 'cues': 'M08 M09', 'effects': 'M10 M11 M12',
    'fixture-personalities': 'M04', 'glossary': 'M22', 'networking': 'M12 M16 M18',
    'palettes': 'M07', 'patching': 'M04 M11', 'quick-start': 'M02 M04 M06 M08 M10',
    'remote-control': 'M17', 'running-the-show': 'M09 M13 M17 M19', 'synergy': 'M12',
    'system-settings': 'M02 M03 M16 M20', 'timelines': 'M13',
    'titan-basics': 'M02 M03 M15 M20', 'titan-net': 'M01 M18', 'titan-reference': 'M15',
}


def canonical(url):
    return urldefrag(url)[0].rstrip('/')


def section(entry):
    path = entry['path']
    if entry['system'] == 'titan':
        return path[0], TITAN_MAP[path[0]].split(), 'module_topics'
    if path[0] == 'grandMA3 User Manual':
        name = path[1] if len(path) > 1 else path[0]
        role = 'module_topics'
        if name in ('Command Syntax and Keywords', 'Plugins'):
            role = 'command_api_reference'
        elif name == 'Device Overview':
            role = 'hardware_reference'
        elif name in ('grandMA3 User Manual', 'New in the Manual', 'About the Help',
                      'Glossary', 'grandMA3 List of Trademarks'):
            role = 'navigation_reference'
        return name, MA_MAP[name].split(), role
    if path[0] == 'grandMA3 Release Notes':
        name = path[1] if len(path) > 1 else path[0]
        return name, ['M21'], 'current_release' if name == 'Release Notes 2.5' else 'historical_release'
    if path[0] == 'grandMA3 Quick Start Guide':
        return path[0], ['M02', 'M04', 'M06', 'M08', 'M10', 'M11', 'M16'], 'tutorial_reference'
    if path[0] == 'Quick Manuals grandMA3 Devices':
        return path[0], ['M01', 'M20'], 'hardware_multilingual_reference'
    raise ValueError('Unmapped section: ' + str(path))


def write_json(name, data):
    (ROOT / name).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')


def build():
    paths = sorted(ROOT.glob('M[0-9][0-9]-*.md'))
    modules = {p.name[:3]: {'file': p.name, 'title': p.read_text().splitlines()[0][2:]} for p in paths}
    assert len(modules) == 22
    records, citations = [], defaultdict(set)
    for path in paths:
        module = path.name[:3]
        body = path.read_text()
        for url in LINK.findall(body):
            citations[canonical(url)].add(path.name)
        previous_sources = []
        for line_no, line in enumerate(body.splitlines(), 1):
            if not re.match(r'^\| M\d\d-', line):
                continue
            assert line_no > 1 and body.splitlines()[line_no - 2].startswith('|'), (
                'Feature row must continue its table', path.name, line_no)
            cols = [c.strip() for c in line.strip('|').split('|')]
            feature_id = cols[0]
            assert re.fullmatch(r'M\d\d-(?:(?:MA|TI)-)?\d\d', feature_id), feature_id
            sources = LINK.findall(line)
            inherited = False
            if '同上' in line:
                sources = list(dict.fromkeys(previous_sources + sources))
                inherited = True
            if module == 'M21' and not sources:
                if '-MA-' in feature_id:
                    sources = ['https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html']
                else:
                    version = '19.0' if int(feature_id.rsplit('-', 1)[1]) <= 6 else '19.2'
                    sources = [f'https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV{version}.pdf']
            assert sources, 'Missing source: ' + feature_id
            previous_sources = sources
            record = {'id': feature_id, 'module': module, 'document': path.name,
                      'line': line_no, 'kind': 'version_delta' if module == 'M21' else 'feature_comparison',
                      'label': cols[1], 'sources': sources, 'source_inherited': inherited,
                      'verification': 'official_documentation_only', 'device_tested': False,
                      'implementation_status': 'not_started'}
            if module == 'M21':
                record['system'] = 'ma3' if '-MA-' in feature_id else 'titan'
                record['description'] = cols[2] if '-TI-' in feature_id else cols[1]
                if '-TI-' in feature_id:
                    record['version'] = cols[1]
                    record['label'] = cols[2]
            else:
                assert len(cols) == 5, (feature_id, cols)
                record.update(ma3=cols[2], titan=cols[3], evidence_note=cols[4])
            record['contains_unverified_equivalence'] = bool(re.search(
                r'未核实|未确认|不假定|不推断|不承诺|不在本次比较中推断', line))
            records.append(record)
    assert len({r['id'] for r in records}) == len(records)
    write_json('feature-catalogue.json', {'research_date': DATE, 'modules': modules, 'features': records,
        'notes': 'Feature comparisons and version deltas are not independent product feature counts; no device tests or implementation claimed.'})

    manual = json.loads((ROOT / 'source-index.json').read_text())
    groups = {}
    for entry in manual:
        group, mapped_modules, role = section(entry)
        entry.update(module_mapping=mapped_modules, coverage_role=role,
                     cited_in=sorted(citations.get(canonical(entry['url']), [])))
        entry['analysis_status'] = 'cited_in_module' if entry['cited_in'] else 'not_individually_cited'
        key = (entry['system'], group)
        g = groups.setdefault(key, {'system': entry['system'], 'section': group, 'modules': mapped_modules,
            'role': role, 'indexed': 0, 'retrieved': 0, 'cited': 0, 'url': entry['url']})
        g['indexed'] += 1
        g['retrieved'] += entry['status'] == 'retrieved'
        g['cited'] += bool(entry['cited_in'])
    write_json('source-index.json', manual)

    api = json.loads((ROOT / 'titan-api-index.json').read_text())
    assert len({e['url'] for e in api}) == len(api)
    for entry in api:
        entry['qualified_name'] = entry['url'].rsplit('/', 1)[-1].removesuffix('.html')
        entry['module_mapping'] = ['M15', 'M17']
        entry['cited_in'] = sorted(citations.get(canonical(entry['url']), []))
        entry['analysis_status'] = 'cited_in_module' if entry['cited_in'] else 'not_individually_cited'
    write_json('titan-api-index.json', api)
    known = {canonical(e['url']) for e in manual + api}
    extras = [{'url': url, 'cited_in': sorted(files), 'coverage_role': 'supplemental_official_source',
               'verification': 'consulted_for_module_claims', 'device_tested': False}
              for url, files in sorted(citations.items()) if url not in known]
    write_json('supplemental-sources.json', extras)
    counts = Counter(r['module'] for r in records)
    summary = {'research_date': DATE, 'modules': len(modules), 'comparison_items': len(records),
               'feature_comparisons': sum(r['kind'] == 'feature_comparison' for r in records),
               'version_deltas': sum(r['kind'] == 'version_delta' for r in records),
               'manual_topics': len(manual), 'manual_retrieved': sum(e['status'] == 'retrieved' for e in manual),
               'manual_cited': sum(bool(e['cited_in']) for e in manual), 'api_topics': len(api),
               'api_cited': sum(bool(e['cited_in']) for e in api), 'supplemental_cited': len(extras),
               'module_counts': dict(sorted(counts.items()))}
    write_json('coverage-summary.json', summary)

    matrix = ['# 统一功能索引', '', f'核查日期：{DATE}。共 {len(records)} 条对照记录，其中 '
              f'{summary["feature_comparisons"]} 条功能比较、{summary["version_deltas"]} 条版本变化；'
              '同一能力可出现在版本变化与功能模块中，因此不能把总数当作互不重叠的产品功能数量。', '',
              '所有条目均为官方文档研究，未实机验证、未实现。完整行为、操作流程和设计判断请点模块链接；'
              '“未核实”不表示“不支持”。原始可检索数据见 [JSON 目录](feature-catalogue.json)。', '',
              '| 模块 | 内容 | 对照记录 |', '| --- | --- | --- |']
    for module, info in modules.items():
        matrix.append(f'| {module} | [{info["title"]}]({info["file"]}) | {counts[module] if counts[module] else "术语、架构与验收建议"} |')
    for module, info in modules.items():
        selected = [r for r in records if r['module'] == module]
        if not selected:
            continue
        matrix += ['', f'## {module} · {info["title"]}', '', f'详细流程与边界：[模块文档]({info["file"]})。', '']
        if module != 'M21':
            matrix += ['| 编号／能力 | grandMA3 | Titan | 证据 |', '| --- | --- | --- | --- |']
            for r in selected:
                sources = '、'.join(f'[来源{i}]({u})' for i, u in enumerate(r['sources'], 1))
                matrix.append(f'| {r["id"]} · {r["label"]} | {r["ma3"]} | {r["titan"]} | {sources} |')
        else:
            matrix += ['| 编号 | 系统 | 变化 | 证据 |', '| --- | --- | --- | --- |']
            for r in selected:
                sources = '、'.join(f'[来源{i}]({u})' for i, u in enumerate(r['sources'], 1))
                matrix.append(f'| {r["id"]} | {r["system"]} | {r["description"]} | {sources} |')
    (ROOT / 'feature-matrix.md').write_text('\n'.join(matrix) + '\n')

    coverage = ['# 官方目录覆盖与研究状态', '', f'核查日期：{DATE}。', '',
        f'已登记 {len(manual):,} 个手册主题，其中 {summary["manual_retrieved"]:,} 个正文已获取，'
        f'{summary["manual_cited"]} 个被模块报告直接引用。另有 {len(api):,} 个 Titan 19.2 API 主题入口和 '
        f'{len(extras)} 个补充官方来源。', '',
        '**目录覆盖不是正文逐页精读率，也不是实机测试率。** `retrieved` 只代表成功获取正文；'
        '`cited_in_module` 只代表被模块报告引用，并不表示该页全部内容或所有 API 都已验证。'
        '未直接引用的专题可能只用于目录定位、关联阅读或后续查阅，统一保留 `not_individually_cited`。', '',
        '本轮全模块整理见 [22 个模块入口](README.md)；功能条目见 [统一索引](feature-matrix.md)；'
        '证据冲突及真实设备验证缺口见 [核查记录](evidence-issues.md)。', '',
        '## 状态字段', '',
        '| 字段 | 含义 |', '| --- | --- |',
        '| indexed | 官方目录存在该主题，已登记标题和链接 |',
        '| retrieved | 正文已获取到临时阅读缓存，不等于已总结全文 |',
        '| cited_in | 引用该页的模块文件；以模块具体说法为结论范围 |',
        '| module_mapping | 该主题类别对应的研究模块；不是该页每项细节已被覆盖的声明 |',
        '| coverage_role | 功能主题、命令 API、硬件、教程、导航或历史版本 |',
        '| displayed_version | 页面正文中显示的版本；可能早于目录集合版本 |', '',
        '## 手册目录逐类对应', '',
        '命令／API、硬件多语言说明和历史版本采用独立参考目录，不逐个当作新功能重复计数。', '']
    for system, label in [('ma3', 'grandMA3'), ('titan', 'Avolites Titan')]:
        coverage += [f'### {label}', '', '| 官方目录 | 研究模块 | 类型 | 索引 | 已获取 | 被模块引用 |', '| --- | --- | --- | ---: | ---: | ---: |']
        for (sysname, _), g in groups.items():
            if sysname != system:
                continue
            refs = '、'.join(f'[{m}]({modules[m]["file"]})' for m in g['modules'])
            coverage.append(f'| [{g["section"]}]({g["url"]}) | {refs} | {g["role"]} | {g["indexed"]} | {g["retrieved"]} | {g["cited"]} |')
        coverage.append('')
    coverage += ['## Titan API 参考', '',
        f'[19.2 API 索引](titan-api-index.json)保留 {len(api):,} 个唯一主题链接，覆盖方法、属性和类型等参考项目；'
        '已剔除无目标的目录分组链接。它不等于这么多个独立功能。API 总体模式和选定播放方法见 M15／M17。', '',
        '## 补充官方来源', '', '| 来源 | 用于模块 |', '| --- | --- |']
    for e in extras:
        refs = '、'.join(f'[{f[:3]}]({f})' for f in e['cited_in'])
        coverage.append(f'| [官方页面]({e["url"]}) | {refs} |')
    coverage += ['', '## 维护方式', '',
        '手工修改模块报告，再运行 `python3 docs/console-research/tools/build_catalogue.py` 重建总表与统计。'
        '脚本会拒绝重复编号、无证据条目、未映射的来源分类和不符合结构的功能行。', '',
        '`source_index.py` 负责索引和按范围获取手册；正文只保存在系统临时目录。索引中的缓存键便于当前会话核查，'
        '不保证临时文件长期存在。版本更新时应明确切换研究基线并复核变化，不把下载成功当作研究完成。', '',
        '两个 Python 文件仅是使用标准库的资料维护工具，不属于 StageMaster 产品业务代码，也不改变 Rust＋TypeScript 技术选型。']
    (ROOT / 'source-coverage.md').write_text('\n'.join(coverage) + '\n')
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    build()
