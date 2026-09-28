"""Author restrained lighting layers in the concert showcase's existing format."""
import json
import os
import sys
import uuid
from pathlib import Path


def apply_layered_looks(project):
    fixtures = project['lighting']['fixtures']
    profiles = {p['id']: p for p in project['lighting']['profiles']}
    roles = {'光束摇头': 'beam', '染色摇头': 'wash', '面光摇头': 'front', '全彩帕灯': 'par'}
    indexed = {}
    counts = dict.fromkeys(roles.values(), 0)
    for fixture in fixtures:
        role = roles[profiles[fixture['profileId']]['model']]
        counts[role] += 1
        indexed[fixture['id']] = (role, counts[role])
    # Primary movement / secondary accent / quiet scenic fill. Off means real zero.
    looks = {
        '环绕波浪': dict(primary=[2,5,8,11], accent=[20,23], wash=[], par=[2,5,8,11], front=[], main=(.015,.25,1), secondary=(.32,.005,1), fill=(.015,.025,.6), floor=.012, fade=1000),
        '对称扇形': dict(primary=[1,3,4,6,7,9,10,12], accent=[], wash=[3,6], par=[2,5,8,11], front=[], main=(.08,.35,1), secondary=(.15,.35,1), fill=(.02,.025,.5), floor=.008, fade=2600),
        '交叉扫动': dict(primary=[13,15,16,18], accent=[3,10], wash=[], par=[2,11], front=[], main=(1,.005,.05), secondary=(1,.12,.005), fill=(.18,.005,.35), floor=.01, fade=2200),
        '中心汇聚': dict(primary=[2,5,8,11], accent=[20,23], wash=[], par=[4,9], front=[4,5], main=(1,.18,.01), secondary=(.015,.08,1), fill=(.01,.035,.45), floor=.012, fade=2500),
    }
    for scene in project['lighting']['scenes']:
        family, step_text = scene['name'].split(' · ')
        look = looks[family]
        elapsed = (int(step_text)-1) * look['fade']
        settings = {}
        primary, accent = [], []
        for fid, (role, number) in indexed.items():
            color, level = look['fill'], 0
            if role == 'beam' and number in look['primary']:
                color, level = look['main'], .24
                primary.append(fid)
            elif role == 'beam' and number in look['accent']:
                color, level = look['secondary'], .055
                accent.append(fid)
            elif role == 'wash' and number in look['wash']:
                color, level = look['fill'], .012
            elif role == 'par' and number in look['par']:
                color, level = look['fill'], look['floor']
            elif role == 'front' and number in look['front']:
                color, level = (1,.55,.25), .008
            settings[fid] = dict(zip(['dimmer','red','green','blue'], [level,*color]))
        for assignment in scene['assignments']:
            target = assignment['target']
            attribute = target['attribute']
            if attribute in settings[target['fixtureId']]:
                assignment['source'] = {'kind':'literal', 'value':{'kind':'normalized','value':round(settings[target['fixtureId']][attribute]*65535)}}
        scene['effects'] = []
        for ids, name, high, low, period, waveform, duty in [
            (primary,'主光束 · 分组追逐',.30,0,4000,'pulse',35),
            (accent,'辅光束 · 缓慢呼吸',.065,0,6400,'smooth',50),
        ]:
            if ids:
                scene['effects'].append({'id':str(uuid.uuid4()),'name':name,'enabled':True,'fixtureIds':ids,'channels':[{'attribute':'dimmer','low':round(low*65535),'high':round(high*65535)}],'periodMs':period,'waveform':waveform,'spreadDegrees':360,'phaseDegrees':round((-elapsed % period)/period*360)%360,'reverse':False,'dutyPercent':duty})
    project['project']['parentRevisionIds'] = [project['project']['revisionId']]
    project['project']['revisionId'] = str(uuid.uuid4())
    project['project']['description'] = '10 × 6 × 6.5 米；地面、独立背墙、五道桁架、80 台通用灯具。暗场留白、主辅光束与低亮度染色分层，四组循环摇头动作。通用光学预演，不代表现场吊挂或电力设计。'


if __name__ == '__main__':
    path = Path(sys.argv[1]).resolve()
    project = json.loads(path.read_text())
    apply_layered_looks(project)
    temporary = path.with_suffix('.writing')
    temporary.write_text(json.dumps(project, ensure_ascii=False, separators=(',',':'))+'\n')
    os.replace(temporary, path)
    print(path)
