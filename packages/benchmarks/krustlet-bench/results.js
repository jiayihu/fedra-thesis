function mean(values) {
  return values.reduce((acc, x) => acc + x, 0) / values.length;
}

function stddev(values) {
  const u = mean(values);
  const se = values.map((x) => (x - u) ** 2).reduce((acc, x) => acc + x, 0);

  return Math.sqrt(se / values.length);
}

const zip = (xs, ys) => xs.map((x, i) => [x, ys[i]]);

// prettier-ignore
const times = [5, 6, 5, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 5, 5, 6, 6, 6, 7, 7, 7, 4, 5, 6, 6, 6, 6, 6, 6, 4, 4, 6, 6, 6, 6, 6, 6, 4, 5, 6, 5, 6, 7, 7, 6]

// const times = zip(start, end).map(([start, end]) => end - start);

console.log(times);

console.log({
  mean: mean(times),
  stddev: stddev(times),
});
