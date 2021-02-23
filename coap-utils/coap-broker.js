const jsonServer = require('json-server');
const WebSocket = require('ws');
const url = require('url');
const app = jsonServer.create();
const router = jsonServer.router({});
const middlewares = jsonServer.defaults();

app.use(middlewares);

app.get(
  '/sensors/rainfall',
  (req, res, next) => {
    if (req.header('Upgrade') === 'websocket') {
      console.log('Websocket');
      next('router');
    } else {
      next();
    }
  },
  (req, res) => {
    res.send(String(Math.random() * 10));
  },
);

app.get(
  '/sensors/flow',
  (req, res, next) => {
    if (req.header('Upgrade') === 'websocket') {
      console.log('Websocket');
      next('router');
    } else {
      next();
    }
  },
  (req, res) => {
    res.send(String(Math.random() * 100));
  },
);

app.use(router);

const server = app.listen(8080, () => {
  console.log('CoAP broker is running');
});

const rainfallWss = new WebSocket.Server({ noServer: true });
const flowWss = new WebSocket.Server({ noServer: true });

rainfallWss.on('connection', function connection(ws) {
  setInterval(() => ws.send(String(Math.random() * 10)), 3000);
});

flowWss.on('connection', function connection(ws) {
  setInterval(() => ws.send(String(Math.random() * 100)), 3000);
});

server.on('upgrade', function upgrade(request, socket, head) {
  const pathname = url.parse(request.url).pathname;

  if (pathname === '/sensors/rainfall') {
    rainfallWss.handleUpgrade(request, socket, head, function done(ws) {
      rainfallWss.emit('connection', ws, request);
    });
  } else if (pathname === '/sensors/flow') {
    flowWss.handleUpgrade(request, socket, head, function done(ws) {
      flowWss.emit('connection', ws, request);
    });
  } else {
    socket.destroy();
  }
});
