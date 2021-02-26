const coap = require('coap');
const gaussian = require('gaussian');

function reply(req, res, data) {
  if (req.headers['Observe'] !== 0) return res.end(data());

  res.write(data());

  const interval = setInterval(() => res.write(data()), 3000 + Math.round(Math.random() * 3000));

  res.on('finish', (err) => clearInterval(interval));
}

function spawnServer(listenCb) {
  const server = coap.createServer();

  server.on('request', function (req, res) {
    const url = req.url;

    console.log(`Request from ${req.rsinfo.address}:${req.rsinfo.port} for ${url}`);

    res.setOption('Content-Format', 'text/plain');

    if (url.startsWith('/well-known/core')) {
      res.end(`
          </sensors/rainfall>;rt="rainfall";if="sensor",
          </sensors/flow>;rt="flow";if="sensor"`);
    } else if (url === '/sensors/rainfall') {
      const distribution = gaussian(0.05, 0.08);

      reply(req, res, () => {
        let sample = distribution.random(1)[0];
        sample = sample < 0 ? -sample : sample;

        return sample.toFixed(7);
      });
    } else if (url === '/sensors/flow') {
      const distribution = gaussian(0.07, 0.09);

      reply(req, res, () => {
        let sample = distribution.random(1)[0];
        sample = sample < 0 ? -sample : sample;

        return sample.toFixed(7);
      });
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
