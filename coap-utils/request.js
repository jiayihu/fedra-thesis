const coap = require('coap');

const HOST = '192.168.1.100';
const PORT = 5683;
const [, , resource, observe] = process.argv;

`${HOST}:${PORT}/${resource}`;

const req = coap.request({
  host: HOST,
  port: PORT,
  pathname: resource,
  observe: !!observe,
});

req.on('response', (res) => {
  res.pipe(process.stdout);
  // res.on('end', () => process.exit(0));
});

req.end();
