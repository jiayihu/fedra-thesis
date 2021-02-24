const WebSocket = require('ws');

const ws = new WebSocket(
  'wss://localhost:3000/exec/default/add/add?command=add&command=1&command=3',
  {
    rejectUnauthorized: false,
  },
);

ws.on('open', () => {
  console.log('Connection open');
});

ws.on('message', (data) => {
  console.log('Data', data.toString());
});
