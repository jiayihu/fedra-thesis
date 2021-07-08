const coap = require('coap');
const yargs = require('yargs/yargs');
const { hideBin } = require('yargs/helpers');

const HOST = '192.168.1.126';
const PORT = 5683;

const argv = yargs(hideBin(process.argv))
  .usage('$0 <resource>', 'Request a CoAP resource', (yargs) => {
    yargs.positional('resource', {
      type: 'string',
      description: 'REST resource',
    });
  })
  .option('observe', {
    type: 'boolean',
  })
  .option('multicast', {
    type: 'boolean',
  }).argv;

const req = coap.request({
  host: HOST,
  port: PORT,
  pathname: argv.resource,
  observe: argv.observe,
  multicast: argv.multicast,
  multicastTimeout: 1000,
  confirmable: argv.multicast ? false : true,
});

req.on('response', (res) => {
  res.pipe(process.stdout);
});

req.end();
