const coap = require('coap');

const server = coap.createServer();

server.on('request', function (req, res) {
  console.log(`Request from ${req.rsinfo.address}:${req.rsinfo.port}`);

  if (req.headers['Observe'] !== 0) return res.end(new Date().toISOString() + '\n');

  const interval = setInterval(function () {
    res.write(new Date().toISOString() + '\n');
  }, 3000);

  res.on('finish', function (err) {
    clearInterval(interval);
  });
});

server.listen(() => {
  console.log('server started');
});
