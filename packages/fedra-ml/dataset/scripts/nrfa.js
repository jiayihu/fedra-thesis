const fs = require('fs');
const util = require('util');
const _ = require('lodash');
const { parse, isSameDay, isValid, format, isAfter } = require('date-fns');

const readFile = util.promisify(fs.readFile);
const writeFile = util.promisify(fs.writeFile);

const STATION_ID = '55002';

Promise.all([loadDailyRainfall(STATION_ID), loadDailyFlow(STATION_ID)])
  .then(([rainFall, dailyFlow]) => {
    let cursor = 0;
    let dataset = [];

    for (let r of rainFall) {
      for (let i = cursor; i < dailyFlow.length; i++) {
        const d = dailyFlow[i];

        if (isAfter(d.date, r.date)) {
          break;
        }

        if (isSameDay(r.date, d.date)) {
          cursor = i;
          dataset.push({
            date: r.date,
            rain: r.rainfall,
            flow: d.flow,
          });
          break;
        }
      }
    }

    return dataset;
  })
  .then((records) => saveRecords(records, './dataset.csv'));

function loadDailyRainfall(name) {
  return readLines(`./${name}_cdr.csv`).then((lines) => {
    return lines.map((line) => {
      const [date, rainfall, _] = line;

      return {
        date: parse(date, 'yyyy-LL-dd', new Date()),
        rainfall: Number(rainfall),
      };
    });
  });
}

function loadDailyFlow(name) {
  return readLines(`./${name}_gdf.csv`).then((lines) => {
    return lines
      .map((line) => {
        const [date, flow] = line;

        return {
          date: parse(date, 'yyyy-LL-dd', new Date()),
          flow: flow ? Number(flow) : null,
        };
      })
      .filter((line) => line.flow !== null);
  });
}

function loadPeakFlow(name) {
  const paddedName = name.padStart(6, '0');

  return readLines(`./${paddedName}.pt`).then((lines) => {
    return lines
      .map((line) => {
        const [date, peak] = line;

        if (!isValid(parse(date, 'dd LLL yyyy', new Date()))) {
          console.log('Invalid date', date);
        }

        return {
          date: parse(date, 'dd LLL yyyy', new Date()),
          peak: Number(peak),
        };
      })
      .filter((line) => line.flow !== null);
  });
}

function readLines(filename) {
  return readFile(filename, 'utf8').then((data) => {
    return data
      .trim()
      .split('\n')
      .map((line) => line.split(','));
  });
}

function saveRecords(records, filename) {
  const lines = records
    .map((record) => {
      return {
        ...record,
        date: format(record.date, 'dd-LL-yyyy'),
      };
    })
    .map((record) => Object.values(record).join(', '));
  const headers = Object.keys(records[0]).join(', ');
  const text = `${headers}\n${lines.join('\n')}`;

  return writeFile(filename, text);
}

function filterBySrcId(dataset, srcId) {
  return dataset.filter((record) => record.srcId === srcId);
}

function sortSrcIds(dataset) {
  return _.countBy(dataset, (x) => x.srcId);
}
