function mean(values) {
  return values.reduce((acc, x) => acc + x, 0) / values.length;
}

function stddev(values) {
  const u = mean(values);
  const se = values.map((x) => (x - u) ** 2).reduce((acc, x) => acc + x, 0);

  return Math.sqrt(se / values.length);
}

// prettier-ignore
const values = [117, 118, 118, 120, 120, 5735, 7168, 8192, 9217, 7703, 8572, 590, 5447, 8622, 8486, 6279, 6330, 6431, 7279, 7330, 4, 5541, 7179, 8203, 9124, 8561]

console.log({
  mean: mean(values),
  stddev: stddev(values),
});
