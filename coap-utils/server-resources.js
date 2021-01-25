const coap = require('coap');

function spawnServer(listenCb) {
  const server = coap.createServer({
    multicastAddress: '224.0.1.187',
  });

  server.on('request', function (req, res) {
    const url = req.url;

    console.log(`Request from ${req.rsinfo.address}:${req.rsinfo.port} for ${url}`);

    if (url.startsWith('/well-known/core')) {
      res.end(`
          </sensors/temp>;rt="oic.r.temperature";if="sensor",
          </sensors/light>;rt="oic.r.light.brightness";if="sensor"`);
    } else if (url === '/sensors/temp') {
      res.end('20\n');
    } else if (url === '/sensors/light') {
      res.end('1000\n');
    } else {
      console.log(`Unknown resource ${url}`);
      res.code = '5.01';
      res.end();
    }
  });

  return server.listen(listenCb);
}

const server1 = spawnServer(() => {
  console.log('server1 started');
});

const server2 = spawnServer(() => {
  console.log('server2 started');
});
