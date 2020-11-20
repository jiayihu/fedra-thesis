const coap = require('coap');

const req = coap.request('coap://192.168.1.100:5683/');

req.on('response', (res) => {
  res.pipe(process.stdout);
  res.on('end', () => process.exit(0));
});

req.end();
