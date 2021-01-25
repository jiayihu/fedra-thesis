const coap = require('coap');

const server = coap.createServer({
  multicastAddress: '224.0.1.187',
});

const server2 = coap.createServer({
  multicastAddress: '224.0.1.187',
});

server.listen(5683, function () {
  console.log('Server 1 is listening');
});

server2.listen(5683, function () {
  console.log('Server 2 is listening');
});

server.on('request', function (msg, res) {
  console.log('Server 1 has received message');
  res.end('Ok');
});

server2.on('request', function (msg, res) {
  console.log('Server 2 has received message');
  res.end('Ok');
});
