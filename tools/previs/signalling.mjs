import { createServer } from 'node:http';
import { timingSafeEqual } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { overrideLogger, MessageHelpers, Messages } from '@epicgames-ps/lib-pixelstreamingcommon-ue5.8';
import { SignallingServer, Logger } from '@epicgames-ps/lib-pixelstreamingsignalling-ue5.8';

// Authentication wraps the upstream protocol; SDP, ICE and reconnect semantics stay upstream.
export function authorize(request, expected, role) {
  const supplied = request.url?.slice(1) ?? '';
  const origin = request.headers.origin;
  const local = request.socket.remoteAddress === '127.0.0.1';
  const validOrigin = role === 'renderer' ? (!origin || ['http://127.0.0.1', '127.0.0.1'].includes(origin)) :
    ['tauri://localhost', 'http://tauri.localhost', 'https://tauri.localhost'].includes(origin);
  const valid = local && validOrigin && /^[a-f0-9]{64}$/.test(supplied) &&
    /^[a-f0-9]{64}$/.test(expected) && timingSafeEqual(Buffer.from(supplied), Buffer.from(expected));
  // Upstream diagnostics include request.url. Never allow credentials into them.
  request.url = '/';
  return valid;
}

function discoverLateRenderer(server) {
  // This local single-renderer viewer queries once, then waits for readiness events.
  // Only viewers that actually received an empty directory need a late notification.
  const waiting = new Set();
  server.playerRegistry.on('added', id => {
    const player = server.playerRegistry.get(id);
    player.protocol.on(Messages.listStreamers.typeName, () => {
      if (!player.subscribedStreamer && !server.streamerRegistry.streamers.some(streamer => streamer.streaming))
        waiting.add(id);
      else waiting.delete(id);
    });
  });
  server.playerRegistry.on('removed', id => waiting.delete(id));
  const notify = () => {
    const message = MessageHelpers.createMessage(Messages.streamerList, {
      ids: server.streamerRegistry.streamers.filter(streamer => streamer.streaming)
        .map(streamer => streamer.streamerId),
    });
    if (!message.ids.length) return;
    for (const id of waiting) {
      const player = server.playerRegistry.get(id);
      waiting.delete(id);
      // Publish once even before subscription acknowledgement; never disturb a live viewer.
      if (player && !player.subscribedStreamer) player.sendMessage(message);
    }
  };
  server.streamerRegistry.on('added', id => {
    server.streamerRegistry.find(id)?.on('id_changed', notify);
  });
  server.streamerRegistry.on('removed', notify);
}

export async function start(rendererToken, viewerToken) {
  if (![rendererToken, viewerToken].every(value => /^[a-f0-9]{64}$/.test(value)))
    throw new Error('缺少预演连接凭据');
  Logger.silent = true;
  overrideLogger({ InitLogging() {}, Debug() {}, Info() {}, Warning() {}, Error() {} });
  const http = () => createServer((_request, response) => { response.writeHead(404); response.end(); });
  const renderer = http(), viewer = http();
  const listen = server => new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => resolve(server.address().port));
  });
  const rendererPort = await listen(renderer);
  const viewerPort = await listen(viewer);
  const options = role => ({
    maxPayload: 256 * 1024, perMessageDeflate: false,
    verifyClient: ({ req }) => authorize(req, role === 'renderer' ? rendererToken : viewerToken, role),
  });
  const server = new SignallingServer({
    streamerPort: 0,
    streamerWsOptions: { server: renderer, port: undefined, ...options('renderer') },
    httpServer: viewer,
    playerWsOptions: options('viewer'),
    maxSubscribers: 1,
    playerKeepaliveTimeout: 15000,
    peerOptions: { iceServers: [] },
    authorizeStreamerId: () => 'StageMaster',
  });
  discoverLateRenderer(server);
  return { rendererPort, viewerPort };
}
if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const ready = await start(process.env.STAGEMASTER_STREAM_RENDERER_TOKEN, process.env.STAGEMASTER_STREAM_VIEWER_TOKEN);
    delete process.env.STAGEMASTER_STREAM_RENDERER_TOKEN;
    delete process.env.STAGEMASTER_STREAM_VIEWER_TOKEN;
    process.stdout.write(`${JSON.stringify(ready)}\n`);
    // Parent owns our lifetime even when it terminates without running shutdown handlers.
    process.stdin.resume();
    process.stdin.on('end', () => process.exit(0));
  } catch {
    process.stderr.write('三维画面连接服务启动失败\n');
    process.exitCode = 1;
    process.exit(1);
  }
}
