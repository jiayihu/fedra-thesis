function mean(values) {
  return values.reduce((acc, x) => acc + x, 0) / values.length;
}

function stddev(values) {
  const u = mean(values);
  const se = values.map((x) => (x - u) ** 2).reduce((acc, x) => acc + x, 0);

  return Math.sqrt(se / values.length);
}

const zip = (xs, ys) => xs.map((x, i) => [x, ys[i]]);

const starts = `
03:29:41.251352 1
03:29:47.798654 2
03:29:54.355035 3
03:30:00.900465 4
03:30:07.448956 5
`
  .trim()
  .split('\n')
  .map((line) => {
    const [time, id] = line.trim().split(' ');
    const matches = Array.from(
      time.matchAll(/(?<hour>\d+):(?<mins>\d+):(?<secs>\d+)\.(?<nanos>\d+)/g),
    );
    const { hour, mins, secs, nanos } = matches[0].groups;

    const date = new Date();
    date.setHours(Number(hour));
    date.setMinutes(mins);
    date.setSeconds(secs);
    date.setMilliseconds(nanos / 1000);

    return {
      id,
      date,
    };
  })
  .reduce((map, record) => {
    map[record.id] = record.date;

    return map;
  }, {});

console.log(starts);

const end = `
1 1618363783632
4 1618363800957
2 1618363787851
5 1618363807511
3 1618363794404
  1618364176000
`
  .trim()
  .split('\n')
  .map((line) => {
    const [id, millis] = line.trim().split(' ');

    const date = new Date(Number(millis));

    return { id, date };
  });

console.log(end);
const times = end.map(({ id, date }) => {
  const start = starts[id];
  const time = date - start;

  console.log(id, time);

  return time;
});

// console.log(times);

// console.log({
//   mean: mean(times),
//   stddev: stddev(times),
// });
