const coap = require('coap');

function reply(req, res, data) {
  if (req.headers['Observe'] !== 0) return res.end(data());

  const interval = setInterval(() => res.write(data()), 3000);

  res.on('finish', (err) => clearInterval(interval));
}

function spawnServer(listenCb) {
  const server = coap.createServer();

  server.on('request', function (req, res) {
    const url = req.url;

    console.log(`Request from ${req.rsinfo.address}:${req.rsinfo.port} for ${url}`);

    if (url.startsWith('/well-known/core')) {
      res.end(`
          </sensors/rainfall>;rt="rainfall";if="sensor",
          </sensors/flow>;rt="flow";if="sensor"`);
    } else if (url === '/sensors/rainfall') {
      reply(req, res, () => String(Math.random() * 10));
    } else if (url === '/sensors/flow') {
      reply(req, res, () => String(Math.random() * 100));
    } else {
      console.log(`Unknown resource ${url}`);
      res.code = '5.01';
      res.end();
    }
  });

  return server.listen(listenCb);
}

spawnServer(() => {
  console.log('server started');
});
