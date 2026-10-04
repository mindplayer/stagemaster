import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { once } from 'node:events';
import WebSocket from 'ws';

// Exercise the installed official service, not a second implementation of its protocol.
async function service(work) {
  const rendererToken = 'd'.repeat(64), viewerToken = 'e'.repeat(64);
  const child = spawn(process.execPath, [new URL('./signalling.mjs', import.meta.url).pathname], {
    env: { ...process.env, STAGEMASTER_STREAM_RENDERER_TOKEN: rendererToken,
      STAGEMASTER_STREAM_VIEWER_TOKEN: viewerToken },
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  const exit = once(child, 'exit');
  const lines = createInterface({ input: child.stdout });
  let diagnostics = '';
  child.stderr.on('data', data => { diagnostics += data; });
  const peers = [];
  try {
    const [line] = await once(lines, 'line');
    const ports = JSON.parse(line);
    function connect(role) {
      const token = role === 'renderer' ? rendererToken : viewerToken;
      const peer = new WebSocket(`ws://127.0.0.1:${ports[`${role}Port`]}/${token}`,
        role === 'viewer' ? { origin: 'tauri://localhost' } : {});
      const messages = [], waiters = [];
      peer.on('message', data => {
        const message = JSON.parse(data);
        messages.push(message);
        for (const waiter of [...waiters]) {
          if (waiter.matches(message)) {
            waiters.splice(waiters.indexOf(waiter), 1);
            clearTimeout(waiter.timeout);
            waiter.resolve(message);
          }
        }
      });
      peer.on('error', () => {});
      peers.push(peer);
      return {
        peer, messages,
        next(matches) {
          return new Promise((resolve, reject) => {
            const waiter = { matches, resolve, timeout: setTimeout(() => {
              waiters.splice(waiters.indexOf(waiter), 1);
              reject(new Error('official signalling discovery did not arrive'));
            }, 1500) };
            waiters.push(waiter);
          });
        },
      };
    }
    await work(connect);
    assert.equal(diagnostics.includes(rendererToken), false);
    assert.equal(diagnostics.includes(viewerToken), false);
  } finally {
    for (const peer of peers) peer.terminate();
    lines.close();
    child.stdin.end();
    await exit;
  }
}

const type = name => message => message.type === name;

test('a waiting viewer discovers a late renderer without another list request or connection',
  { timeout: 6000 }, () => service(async connect => {
    const viewer = connect('viewer');
    await viewer.next(type('config'));
    const empty = viewer.next(type('streamerList'));
    viewer.peer.send(JSON.stringify({ type: 'listStreamers' }));
    assert.deepEqual((await empty).ids, []);
    // The local frontend queries once and then waits; no timer or second query is involved.
    const available = viewer.next(type('streamerList'));
    const renderer = connect('renderer');
    await renderer.next(type('identify'));
    renderer.peer.send(JSON.stringify({ type: 'endpointId', id: 'DefaultStreamer' }));
    assert.deepEqual((await available).ids, ['StageMaster']);
    const readyAgain = renderer.next(type('endpointIdConfirm'));
    renderer.peer.send(JSON.stringify({ type: 'endpointId', id: 'DefaultStreamer' }));
    await readyAgain;
    const synchronized = viewer.next(type('pong'));
    viewer.peer.send(JSON.stringify({ type: 'ping', time: 1 }));
    await synchronized;
    assert.equal(viewer.messages.filter(type('streamerList')).length, 2,
      'an identified renderer must announce ready only once while subscription is pending');
    const subscribed = renderer.next(type('playerConnected'));
    viewer.peer.send(JSON.stringify({ type: 'subscribe', streamerId: 'StageMaster' }));
    await subscribed;
    assert.equal(viewer.peer.readyState, WebSocket.OPEN);
    const confirmed = renderer.next(type('endpointIdConfirm'));
    renderer.peer.send(JSON.stringify({ type: 'endpointId', id: 'DefaultStreamer' }));
    await confirmed;
    assert.equal(viewer.messages.filter(type('streamerList')).length, 2,
      'a live subscriber must not receive another discovery message');
  }));

test('a renderer ready before the initial directory request cannot double-discover a viewer',
  { timeout: 6000 }, () => service(async connect => {
    const viewer = connect('viewer');
    await viewer.next(type('config'));
    const renderer = connect('renderer');
    await renderer.next(type('identify'));
    const identified = renderer.next(type('endpointIdConfirm'));
    renderer.peer.send(JSON.stringify({ type: 'endpointId', id: 'DefaultStreamer' }));
    await identified;
    const synchronized = viewer.next(type('pong'));
    viewer.peer.send(JSON.stringify({ type: 'ping', time: 2 }));
    await synchronized;
    assert.equal(viewer.messages.filter(type('streamerList')).length, 0,
      'a viewer not yet waiting uses its initial directory response, not an unsolicited duplicate');
    const available = viewer.next(type('streamerList'));
    viewer.peer.send(JSON.stringify({ type: 'listStreamers' }));
    assert.deepEqual((await available).ids, ['StageMaster']);
    const subscribed = renderer.next(type('playerConnected'));
    viewer.peer.send(JSON.stringify({ type: 'subscribe', streamerId: 'StageMaster' }));
    await subscribed;
    assert.equal(viewer.messages.filter(type('streamerList')).length, 1);
    assert.equal(viewer.peer.readyState, WebSocket.OPEN);
  }));

test('a cancelled waiting viewer stays closed; a new viewer discovers the current renderer',
  { timeout: 6000 }, () => service(async connect => {
    const cancelled = connect('viewer');
    await cancelled.next(type('config'));
    const empty = cancelled.next(type('streamerList'));
    cancelled.peer.send(JSON.stringify({ type: 'listStreamers' }));
    assert.deepEqual((await empty).ids, []);
    const closed = once(cancelled.peer, 'close');
    cancelled.peer.close();
    await closed;
    const renderer = connect('renderer');
    await renderer.next(type('identify'));
    const identified = renderer.next(type('endpointIdConfirm'));
    renderer.peer.send(JSON.stringify({ type: 'endpointId', id: 'DefaultStreamer' }));
    await identified;
    const viewer = connect('viewer');
    await viewer.next(type('config'));
    const available = viewer.next(type('streamerList'));
    viewer.peer.send(JSON.stringify({ type: 'listStreamers' }));
    assert.deepEqual((await available).ids, ['StageMaster']);
    assert.equal(cancelled.peer.readyState, WebSocket.CLOSED);
  }));
