const coap = require('coap');

const HOST = 'coap://192.168.1.100';
const PORT = 5683;
const [, , resource] = process.argv;

let req;

switch (resource) {
  default:
    req = coap.request(`${HOST}:${PORT}/${resource}`);
}

req.on('response', (res) => {
  res.pipe(process.stdout);
  res.on('end', () => process.exit(0));
});

req.end();
