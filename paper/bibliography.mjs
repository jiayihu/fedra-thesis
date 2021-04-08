import fs from 'fs';
import { promisify } from 'node:util';

const readFile = promisify(fs.readFile);

const loadCitations = readFile('./text.txt', 'utf-8').then((file) => {
  const citations = Array.from(file.matchAll(/cite\{(?<cite>[\w-]+)\}/g)).map(
    (match) => match.groups.cite,
  );

  return citations;
});

const loadBibitems = readFile('./bibitems.txt', 'utf-8').then((file) => {
  const bibitems = file
    .trim()
    .split('\n')
    .filter((line) => line.trim().length > 1)
    .map((line) => {
      const cite = line.match(/bibitem\{(?<cite>[\w-]+)\}/).groups.cite;

      return {
        cite,
        bibitem: line,
      };
    });

  return bibitems;
});

Promise.all([loadCitations, loadBibitems])
  .then(([citations, bibitems]) => {
    const orderedBibitems = new Map();
    const bibitemsById = bibitems.reduce((map, item) => {
      map.set(item.cite, item.bibitem);

      return map;
    }, new Map());

    citations.forEach((citation) => {
      orderedBibitems.set(citation, bibitemsById.get(citation));
    });

    return orderedBibitems;
  })
  .then((orderedBibitems) => {
    const latex = Array.from(orderedBibitems.values()).join('\n\n');

    console.log(latex);
  });
