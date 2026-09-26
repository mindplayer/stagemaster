import { test } from 'node:test';
import assert from 'node:assert/strict';
import { authorize } from './signalling.mjs';
const token = 'a'.repeat(64);
function request(origin, url = `/${token}`, remoteAddress = '127.0.0.1') {
  return { url, headers: origin ? { origin } : {}, socket: { remoteAddress } };
}
test('viewer credentials cannot authorize another origin or the renderer', () => {
  for (const origin of ['https://attacker.example', 'null', undefined])
    assert.equal(authorize(request(origin), token, 'viewer'), false);
  assert.equal(authorize(request('tauri://localhost'), token, 'renderer'), false);
  assert.equal(authorize(request('tauri://localhost', `/${token}`, '192.168.1.2'), token, 'viewer'), false);
  assert.equal(authorize(request('tauri://localhost', '/wrong'), token, 'viewer'), false);
  assert.equal(authorize(request(undefined), 'b'.repeat(64), 'renderer'), false);
});
test('authorized paths are removed before upstream logging', () => {
  for (const [role, origin] of [['renderer', undefined], ['viewer', 'tauri://localhost']]) {
    const req = request(origin);
    assert.equal(authorize(req, token, role), true);
    assert.equal(req.url, '/');
  }
});

test('actual loopback service rejects unauthorized upgrades and negotiates official config', { timeout: 6000 }, async () => {
  const { spawn } = await import('node:child_process');
  const { createInterface } = await import('node:readline');
  const { once } = await import('node:events');
  const { default: WebSocket } = await import('ws');
  const rendererToken = 'b'.repeat(64);
  const child = spawn(process.execPath, [new URL('./signalling.mjs', import.meta.url).pathname], {
    env: { ...process.env, STAGEMASTER_STREAM_RENDERER_TOKEN: rendererToken, STAGEMASTER_STREAM_VIEWER_TOKEN: token },
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  let output = '';
  child.stdout.on('data', chunk => { output += chunk; });
  child.stderr.on('data', chunk => { output += chunk; });
  const lines = createInterface({ input: child.stdout });
  const peers = [];
  try {
    const [line] = await once(lines, 'line');
    const { rendererPort, viewerPort } = JSON.parse(line);
    for (const [port, key, origin] of [
      [viewerPort, token, 'https://attacker.example'],
      [viewerPort, 'c'.repeat(64), 'tauri://localhost'],
      [rendererPort, token, undefined],
    ]) {
      const peer = new WebSocket(`ws://127.0.0.1:${port}/${key}`, origin ? { origin } : {});
      peers.push(peer);
      const [error] = await once(peer, 'error');
      assert.match(error.message, /401/);
    }
    for (const [port, key, origin] of [
      [rendererPort, rendererToken, undefined], [viewerPort, token, 'tauri://localhost'],
    ]) {
      const peer = new WebSocket(`ws://127.0.0.1:${port}/${key}`, origin ? { origin } : {});
      peers.push(peer);
      const config = await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('configuration handshake timed out')), 2000);
        peer.on('message', payload => {
          const message = JSON.parse(payload);
          if (message.type === 'config') { clearTimeout(timeout); resolve(message); }
        });
      });
      assert.deepEqual(config.peerConnectionOptions.iceServers, []);
    }
    assert.equal(output.includes(token), false);
    assert.equal(output.includes(rendererToken), false);
  } finally {
    for (const peer of peers) peer.terminate();
    lines.close();
    child.stdin.end();
    await once(child, 'exit');
  }
});
