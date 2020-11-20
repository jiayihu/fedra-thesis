const coap = require('coap');

const server = coap.createServer();

server.on('request', (req, res) => {
  res.end('Hello ' + req.url.split('/')[1] + '\n');
});

server.listen(() => {});
