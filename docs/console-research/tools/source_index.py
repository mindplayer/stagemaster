"""Index official manual topics and cache selected pages for research.

Only titles, URLs, hierarchy and retrieval metadata are written to the repository.
Full source text is kept in a temporary cache, not published as project content.
"""
import argparse
import concurrent.futures
from datetime import date
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import tempfile
import time
from urllib.parse import urljoin, urlsplit
from urllib.request import Request, urlopen
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
CACHE = Path(tempfile.gettempdir()) / 'stagemaster-console-research'
MA_BASE = 'https://help.malighting.com/grandMA3/2.5/'
AV_BASE = 'https://manual.avolites.com/'


def retrieve(url):
    CACHE.mkdir(parents=True, exist_ok=True)
    key = hashlib.sha256(url.encode()).hexdigest()
    path = CACHE / (key + '.html')
    if path.exists():
        return path.read_text()
    request = Request(url, headers={'User-Agent': 'StageMasterResearch/1.0 (official documentation review)'})
    with urlopen(request, timeout=25) as response:
        result = response.read().decode('utf-8')
    path.write_text(result)
    return result


class Article(HTMLParser):
    def __init__(self):
        super().__init__()
        self.depth = 0
        self.active = False
        self.skip = 0
        self.parts = []
        self.headings = []
        self.heading = None
        self.links = []
        self.anchor = None

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if not self.active and tag == 'article':
            self.active = True
            self.depth = 1
            return
        if not self.active:
            return
        if tag not in ('img', 'br', 'hr', 'input', 'meta', 'link', 'source', 'wbr'):
            self.depth += 1
        if tag in ('script', 'style', 'svg'):
            self.skip += 1
        if tag in ('p', 'li', 'tr', 'div', 'br', 'h1', 'h2', 'h3', 'h4', 'h5'):
            self.parts.append('\n')
        if tag in ('td', 'th'):
            self.parts.append(' | ')
        if tag in ('h1', 'h2', 'h3', 'h4', 'h5'):
            self.heading = [tag, attrs.get('id', ''), []]
        if tag == 'a':
            self.anchor = [attrs.get('href', ''), []]

    def handle_endtag(self, tag):
        if not self.active:
            return
        if tag in ('script', 'style', 'svg'):
            self.skip = max(0, self.skip - 1)
        if self.heading and tag == self.heading[0]:
            self.headings.append({'level': tag, 'id': self.heading[1], 'text': ''.join(self.heading[2]).strip()})
            self.heading = None
        if tag == 'a' and self.anchor:
            self.links.append({'href': self.anchor[0], 'text': ''.join(self.anchor[1]).strip()})
            self.anchor = None
        if tag in ('p', 'li', 'tr', 'div', 'h1', 'h2', 'h3', 'h4', 'h5'):
            self.parts.append('\n')
        if tag not in ('img', 'br', 'hr', 'input', 'meta', 'link', 'source', 'wbr'):
            self.depth -= 1
        if self.depth <= 0:
            self.active = False

    def handle_data(self, data):
        if not self.active or self.skip:
            return
        self.parts.append(data)
        if self.heading:
            self.heading[2].append(data)
        if self.anchor:
            self.anchor[1].append(data)

    def text(self):
        return '\n'.join(' '.join(line.split()) for line in ''.join(self.parts).splitlines() if line.strip())


def make_index():
    script = retrieve(MA_BASE + '_webHelpScripts/Master/toc_nav.js')
    start = script.index('[{"id":')
    tree, _ = json.JSONDecoder().raw_decode(script[start:])
    entries = []

    def visit(nodes, parents):
        for node in nodes:
            slug = node.get('vals', {}).get('e')
            if slug:
                entries.append({'system': 'ma3', 'version': '2.5', 'title': node['t'],
                                'path': parents + [node['t']], 'slug': slug,
                                'url': MA_BASE + 'HTML/' + slug + '.html', 'status': 'indexed'})
            visit(node.get('c', []), parents + [node['t']])
    visit(tree, [])
    sitemap = ET.fromstring(retrieve(AV_BASE + 'sitemap.xml'))
    for loc in sitemap.findall('.//{*}loc'):
        url = loc.text
        path = urlsplit(url).path
        if not path.startswith('/docs/') or re.match(r'/docs/(?:\d|next/)', path):
            continue
        slug = path.removeprefix('/docs/').strip('/')
        entries.append({'system': 'titan', 'version': '19.0 manual; 19.2 release delta',
                        'title': slug.rsplit('/', 1)[-1] or 'Introduction',
                        'path': slug.split('/') if slug else ['Introduction'],
                        'slug': slug, 'url': url, 'status': 'indexed'})
    unique = {entry['url'].rstrip('/'): entry for entry in entries}
    entries = list(unique.values())
    index_path = ROOT / 'source-index.json'
    if index_path.exists():
        previous = {e['url'].rstrip('/'): e for e in json.loads(index_path.read_text())}
        identity = {'system', 'version', 'path', 'slug', 'url'}
        for entry in entries:
            old = previous.get(entry['url'].rstrip('/'))
            if old and old.get('version') == entry['version']:
                entry.update({key: value for key, value in old.items() if key not in identity})
    index_path.write_text(json.dumps(entries, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'indexed': len(entries), 'ma3': sum(e['system'] == 'ma3' for e in entries),
                      'titan': sum(e['system'] == 'titan' for e in entries), 'cache': str(CACHE)}))


def fetch_one(entry):
    parser = Article()
    parser.feed(retrieve(entry['url']))
    text = parser.text()
    if not text or not parser.headings:
        raise ValueError('Article extraction empty: ' + entry['url'])
    key = hashlib.sha256(entry['url'].encode()).hexdigest()
    title = next((h['text'] for h in parser.headings if h['level'] == 'h1' and h['text']), None)
    title = title or entry.get('title') or entry.get('path', ['Untitled'])[-1]
    result = {'url': entry['url'], 'title': title,
              'headings': parser.headings, 'links': parser.links, 'text': text}
    (CACHE / (key + '.json')).write_text(json.dumps(result, ensure_ascii=False, indent=2))
    displayed = re.search(r'\bVersion\s+(\d+(?:\.\d+)+)', text)
    return {'title': result['title'], 'headings': parser.headings, 'chars': len(text),
            'displayed_version': displayed.group(1) if displayed else None,
            'cache_key': key, 'status': 'retrieved', 'retrieved_date': date.today().isoformat()}


def main():
    args = argparse.ArgumentParser()
    args.add_argument('--index', action='store_true')
    args.add_argument('--fetch', choices=['ma3', 'titan', 'both'])
    args.add_argument('--match', default='')
    args.add_argument('--limit', type=int, default=10000)
    opts = args.parse_args()
    if opts.index:
        make_index()
    if opts.fetch:
        path = ROOT / 'source-index.json'
        entries = json.loads(path.read_text())
        selected = [e for e in entries if (opts.fetch == 'both' or e['system'] == opts.fetch)
                    and (not opts.match or re.search(opts.match, ' / '.join(e['path']) + ' ' + e['slug']))
                    and e['status'] != 'retrieved'][:opts.limit]
        print('Fetching', len(selected), flush=True)
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            futures = {pool.submit(fetch_one, e): e for e in selected}
            for number, future in enumerate(concurrent.futures.as_completed(futures), 1):
                entry = futures[future]
                try:
                    entry.update(future.result())
                except Exception as exc:
                    entry.update(status='retrieval_failed', error=str(exc))
                if number % 20 == 0 or number == len(selected):
                    path.write_text(json.dumps(entries, ensure_ascii=False, indent=2) + '\n')
                    print(number, '/', len(selected), 'saved', flush=True)
        print('Done', flush=True)


if __name__ == '__main__':
    main()
