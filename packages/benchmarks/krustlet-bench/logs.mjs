import https from 'https';
import { stdout } from 'node:process';
import { promisify } from 'node:util';

function streamToString(stream) {
  const chunks = [];
  return new Promise((resolve, reject) => {
    stream.on('data', (chunk) => chunks.push(Buffer.from(chunk)));
    stream.on('error', (err) => reject(err));
    stream.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
  });
}

const getLogs = (i) =>
  new Promise((resolve, reject) => {
    https.get(
      {
        hostname: 'rasp-4',
        port: 3000,
        path: `/containerLogs/default/hello-wasi-${i}/hello-wasi-${i}`,
        rejectUnauthorized: false,
      },
      (res) => resolve(res),
    );
  })
    .then((res) => streamToString(res))
    .then((log) => console.log(i, log));

Array.from({ length: 120 }, (_, i) => i).reduce(
  (acc, i) => acc.then(() => getLogs(i)),
  Promise.resolve(),
);
