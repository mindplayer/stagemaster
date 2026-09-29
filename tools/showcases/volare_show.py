"""Author a portable music-led show on the existing curved theatre, without device output."""
import argparse
import copy
import hashlib
import json
import shutil
from pathlib import Path
from volare_looks import LOOKS, fixture_roles, make_scene, settings, uid

ROOT = Path(__file__).resolve().parents[2]


def position_values(scene):
    result = {}
    for value in scene['assignments']:
        fid, attr = value['target']['fixtureId'], value['target']['attribute']
        if attr in ['pan', 'tilt']:
            result.setdefault(fid, {})[attr] = value['source']['value']['value']
    return result


def build(source, asset):
    project = copy.deepcopy(source)
    roles = fixture_roles(project)
    originals = project['lighting']['scenes']
    families = {}
    for scene in originals:
        families.setdefault(scene['name'].split(' · ')[0], []).append(scene)
    poses = position_values(families[LOOKS[0][2]][0])
    authored, markers, schedule = [], [], []
    for i, look in enumerate(LOOKS):
        time, title, _, _, color, _, mood = look
        levels = settings(look, roles)
        # Only reposition a dark head. It is prepared a scene before its next entrance.
        future = LOOKS[min(i + 1, len(LOOKS) - 1)]
        family = families[future[2]]
        desired = position_values(family[(i * 3) % len(family)])
        for fid in poses:
            if levels[fid][0] == 0:
                poses[fid] = desired[fid]
        scene, levels = make_scene(i, roles, originals[0], poses)
        authored.append(scene)
        markers.append(dict(id=uid(), name=title, timeMs=time, sceneId=scene['id']))
        schedule.append(dict(timeMs=time, name=title, color=color, mood=mood,
            possibleLitFixtures=sum(level > 0 for level, _ in levels.values()),
            leadBeams=look[3]))
    # Additional accents are neutral music marks; they do not restart effects.
    for time in [2510, 4570, 6410, 8730, 10770, 13140, 15710, 18290, 20360, 22440, 24210, 26050, 27060]:
        markers.append(dict(id=uid(), name='节奏重音', timeMs=time, sceneId=None))
    markers.sort(key=lambda marker: marker['timeMs'])
    def duration(ms):
        return dict(ticks=str(ms), ticksPerSecond='1000')
    steps = []
    for i, (scene, look) in enumerate(zip(authored, LOOKS)):
        end = LOOKS[i+1][0] if i+1 < len(LOOKS) else 30000
        steps.append(dict(id=uid(), name=look[1], number=str(i+1), sceneId=scene['id'],
            actionIds=[], delay=duration(0), fade=duration(0),
            advance=dict(kind='after', wait=duration(end-look[0]))))
    project['lighting']['scenes'] = authored
    project['lighting']['sequences'] = [dict(id=uid(), name='Volare · 灯光独立检查（不带音乐）',
        tracking='isolated', repeat='once', release='profile-defaults', steps=steps)]
    project['project'].update(id=uid(), revisionId=uid(), parentRevisionIds=[],
        name='Volare · 蓝金之夜', description=source['project']['description'] +
        ' 音乐秀：使用30秒官方试听，17段蓝金／珊瑚灯光编排；对称追逐、暗部换位、停顿留白和末尾渐暗。灯位、座席及配适保留。')
    project['media'] = dict(systems=[], objects=[], audioEditing=dict(
        asset=asset, inMs=0, outMs=30000, markers=markers))
    for capability in ['media.audio-editing', 'lighting.effects.basic', 'lighting.effects.keyframes']:
        if not any(r['key'] == capability for r in project['requires']):
            project['requires'].append(dict(key=capability, version=1))
    assert project['stage'] == source['stage']
    for key in ['fixtures', 'profiles', 'patches', 'groups']:
        assert project['lighting'][key] == source['lighting'][key]
    return project, schedule


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'data/showcases/圆弧剧场.project.json')
    parser.add_argument('--music', type=Path, default=ROOT/'data/AUDIO-001/Gipsy Kings - Volare（官方试听）.mp3')
    parser.add_argument('--output', type=Path, default=ROOT/'data/SHOW-003/Volare·蓝金之夜.project.json')
    args = parser.parse_args()
    if args.output.exists():
        parser.error('目标工程已存在，拒绝覆盖用户编辑')
    digest = hashlib.sha256(args.music.read_bytes()).hexdigest()
    if digest != 'cc5a3dff10abef5380a6a79b6d6ae172533803c5f4cabaf055a6d5e251be3827':
        parser.error('编排时间只适用于已核验的30秒官方试听，不能套用其他版本')
    asset = dict(digest=digest, durationMs=30000, extension='mp3', fileName=args.music.name)
    project, schedule = build(json.loads(args.source.read_text()), asset)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    assets = Path(str(args.output)+'.assets')
    assets.mkdir(exist_ok=True)
    shutil.copyfile(args.music, assets/(digest+'.mp3'))
    args.output.write_text(json.dumps(project, ensure_ascii=False, separators=(',', ':'))+'\n')
    args.output.with_suffix('.schedule.json').write_text(json.dumps(schedule, ensure_ascii=False, indent=2)+'\n')
    print(args.output)
    print(f'{len(LOOKS)} 段灯光，{len(project["media"]["audioEditing"]["markers"])} 个卡点')


if __name__ == '__main__':
    main()
