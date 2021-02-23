const jsonServer = require('json-server');
const server = jsonServer.create();
const router = jsonServer.router({});
const middlewares = jsonServer.defaults();

server.use(middlewares);

server.get('/sensors/rainfall', (req, res) => {
  res.send(String(Math.random() * 10));
});

server.get('/sensors/flow', (req, res) => {
  res.send(String(Math.random() * 100));
});

server.use(router);

server.listen(8080, () => {
  console.log('CoAP broker is running');
});
