const coap = require('coap');

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
      res.end('20\n');
    } else if (url === '/sensors/flow') {
      res.end('1000\n');
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
