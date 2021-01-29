const fs = require('fs');
const util = require('util');
const _ = require('lodash');

const readFile = util.promisify(fs.readFile);
const writeFile = util.promisify(fs.writeFile);

// loadDataset()
//   .then((dataset) => filterBySrcId(dataset, '926'))
//   .then((records) => {
//     return saveRecords(records, 'rain-926.csv');
//   });

loadDataset().then((dataset) => console.log(sortSrcIds(dataset)));

function loadDataset() {
  return readFile('./midas_rainsub_200501-200504.txt', 'utf8').then((data) => {
    const dataset = data
      .split('\n')
      .map((line) => {
        return line.split(', ');
      })
      .map((line) => {
        const [
          time,
          idType,
          id,
          domainName,
          srcId,
          precipit,
          recStInd,
          metoStmpTime,
          stmpETime,
        ] = line;

        return {
          time,
          srcId,
          precipit,
        };
      });

    return dataset;
  });
}

function saveRecords(records, filename) {
  const lines = records.map((record) => Object.values(record).join(', '));
  const text = lines.join('\n');

  return writeFile(filename, text);
}

function filterBySrcId(dataset, srcId) {
  return dataset.filter((record) => record.srcId === srcId);
}

function sortSrcIds(dataset) {
  return _.countBy(dataset, (x) => x.srcId);
}
