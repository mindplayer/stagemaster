"""Lighting direction for the 30-second official Volare preview, not a beat detector."""
import copy
import uuid

COLORS = {
    '海蓝': (.02, .28, 1), '青蓝': (.035, .65, 1),
    '暖金': (1, .34, .045), '珊瑚': (1, .035, .095),
    '暖白': (1, .66, .32), '紫红': (.55, .015, .38),
}
# Music onsets are from this exact preview, not timestamps for the full song.
# time / title / pose family / lead beam numbers / lead color / lead level / mood
LOOKS = [
    (0, '金色序幕', '中心汇聚', [20, 23], '暖金', .055, 'intro'),
    (1760, '蓝色起势', '对称扇形', [2, 5, 8, 11], '青蓝', .28, 'verse'),
    (3830, '暖金应答', '环绕波浪', [1, 4, 9, 12], '暖金', .28, 'verse'),
    (5880, '海风展开', '对称扇形', [2, 3, 6, 7, 10, 11], '青蓝', .30, 'open'),
    (7980, '金色流动', '环绕波浪', [1, 4, 5, 8, 9, 12], '暖金', .30, 'open'),
    (10020, '珊瑚交织', '交叉扫动', [13, 15, 16, 18], '珊瑚', .29, 'cross'),
    (12090, '蓝金对话', '交叉扫动', [2, 5, 8, 11], '青蓝', .28, 'cross'),
    (13620, '收束', '中心汇聚', [20, 23], '暖金', .045, 'soft'),
    (14160, '留白呼吸', '中心汇聚', [], '暖金', 0, 'break'),
    (15170, '再次起飞', '对称扇形', [1, 3, 6, 7, 10, 12], '暖金', .32, 'open'),
    (17260, '蓝色回应', '环绕波浪', [2, 4, 5, 8, 9, 11], '青蓝', .32, 'open'),
    (19330, '盛放', '对称扇形', [1, 3, 4, 6, 7, 9, 10, 12], '暖金', .34, 'peak'),
    (21390, '蓝金交响', '对称扇形', [1, 3, 4, 6, 7, 9, 10, 12], '青蓝', .34, 'peak'),
    (23440, '最后的舞步', '交叉扫动', [13, 14, 15, 16, 17, 18], '珊瑚', .30, 'cross'),
    (25510, '金色回望', '中心汇聚', [2, 5, 8, 11], '暖金', .29, 'open'),
    (27570, '余晖谢幕', '对称扇形', [1, 3, 4, 6, 7, 9, 10, 12], '暖金', .22, 'outro'),
    (29950, '音乐结束 · 全暗', '对称扇形', [], '暖金', 0, 'black'),
]


def uid():
    return str(uuid.uuid4())


def normalized(value):
    return round(max(0, min(1, value)) * 65535)


def fixture_roles(project):
    models = {p['id']: p['model'] for p in project['lighting']['profiles']}
    roles = {'光束摇头': 'beam', '染色摇头': 'wash', '面光摇头': 'front', '全彩帕灯': 'par'}
    counts = dict.fromkeys(roles.values(), 0)
    result = {}
    for fixture in project['lighting']['fixtures']:
        role = roles[models[fixture['profileId']]]
        counts[role] += 1
        result[fixture['id']] = role, counts[role]
    assert counts == {'beam': 24, 'wash': 16, 'front': 8, 'par': 32}, counts
    return result


def settings(look, roles):
    _, _, _, lead, color, high, mood = look
    values = {}
    for fid, (role, number) in roles.items():
        level, tint = 0, '海蓝'
        if role == 'beam' and number in lead:
            level, tint = high, color
        elif role == 'beam' and number in [20, 23] and mood in ['open', 'peak', 'cross']:
            level, tint = .040 if mood == 'peak' else .025, '青蓝' if color == '暖金' else '暖金'
        elif role == 'par' and number in [2, 4, 6, 7, 9, 11]:
            level = .004 if mood in ['break', 'soft'] else .010
            tint = '海蓝' if color == '暖金' else '暖金'
        elif role == 'par' and number in [14, 17, 20, 23]:
            level, tint = .0025 if mood == 'break' else .005, '暖金'
        elif role == 'front' and number in [3, 6]:
            level, tint = .006 if mood != 'peak' else .009, '暖白'
        elif role == 'wash' and number in [3, 6] and mood in ['open', 'peak', 'outro']:
            level, tint = .010, '海蓝'
        if mood == 'black':
            level = 0
        values[fid] = (level, COLORS[tint])
    return values


def envelope(name, ids, high, period, mode='chase', spread=0):
    if mode == 'in':
        points = [(0, 0), (8800, high), (9999, high)]
    elif mode == 'out':
        points = [(0, high), (1200, high), (8700, 0), (9999, 0)]
    elif mode == 'breathe':
        points = [(0, high * .2), (5000, high), (9999, high * .2)]
    else:
        points = [(0, 0), (700, high), (2200, high), (4900, 0), (9999, 0)]
    return dict(id=uid(), name=name, enabled=True, fixtureIds=ids,
        channels=[dict(attribute='dimmer', keyframes=[dict(position=p,
            value=normalized(v), transition='smooth') for p, v in points])],
        periodMs=period, waveform='keyframes', spreadDegrees=spread,
        phaseDegrees=0, reverse=False, dutyPercent=50)


def make_scene(index, roles, template, poses):
    look = LOOKS[index]
    start, title, _, lead, _, high, mood = look
    levels = settings(look, roles)
    scene = copy.deepcopy(template)
    scene.update(id=uid(), name=f'{index + 1:02} · {title}', effects=[])
    for assignment in scene['assignments']:
        fid, attribute = assignment['target']['fixtureId'], assignment['target']['attribute']
        if attribute in ['pan', 'tilt']:
            value = poses[fid][attribute]
        else:
            level, rgb = levels[fid]
            value = normalized(dict(zip(['dimmer', 'red', 'green', 'blue'], [level, *rgb]))[attribute])
        assignment['source'] = {'kind': 'literal', 'value': {'kind': 'normalized', 'value': value}}
    if mood in ['intro', 'outro']:
        grouped = {}
        for fid, (level, _) in levels.items():
            if level:
                grouped.setdefault(level, []).append(fid)
        end = LOOKS[index + 1][0]
        for level, ids in grouped.items():
            scene['effects'].append(envelope('渐入' if mood == 'intro' else '渐暗',
                ids, level, end - start + 40, 'in' if mood == 'intro' else 'out'))
    elif lead:
        # Mirrored pairs share phase: symmetric spatial chase instead of all-on flashing.
        ordered = sorted((fid for fid, (role, n) in roles.items() if role == 'beam' and n in lead),
                         key=lambda fid: roles[fid][1])
        for pair in range((len(ordered) + 1) // 2):
            ids = list(dict.fromkeys([ordered[pair], ordered[-1-pair]]))
            effect = envelope('对称光束 · 柔和追逐', ids, high,
                2068 if mood != 'soft' else 3000, 'breathe' if mood == 'soft' else 'chase')
            effect['phaseDegrees'] = round(pair * 360 / ((len(ordered) + 1) // 2)) % 360
            scene['effects'].append(effect)
        accents = [fid for fid, (role, n) in roles.items()
                   if role == 'beam' and n not in lead and levels[fid][0] > 0]
        if accents:
            scene['effects'].append(envelope('边缘辅光 · 慢呼吸', accents,
                levels[accents[0]][0], 4136, 'breathe'))
    return scene, levels
