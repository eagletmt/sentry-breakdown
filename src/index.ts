import Papa from 'papaparse';
const csvInput = document.getElementById('csv-input')!;
csvInput.addEventListener('change', handleCsvInput);

function handleCsvInput(this: HTMLInputElement) {
  const file = this.files![0];
  Papa.parse(file, {
    header: true,
    skipEmptyLines: true,
    complete: onCsvParseComplete,
  });
}

interface Row {
  [key: string]: string;
}

function onCsvParseComplete(results: Papa.ParseResult<Row>) {
  if (results.data.length === 0) {
    return;
  }
  const header = document.createElement('tr');
  const fields = Object.keys(results.data[0]);
  for (const key of fields) {
    const th = document.createElement('th');
    th.appendChild(document.createTextNode(key));
    header.appendChild(th);
  }
  const thead = document.createElement('thead');
  thead.appendChild(header);

  const tbody = document.createElement('tbody');
  for (const row of results.data) {
    const tr = document.createElement('tr');
    for (const key of fields) {
      const td = document.createElement('td');
      td.appendChild(document.createTextNode(row[key]));
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }

  const table = document.createElement('table');
  table.appendChild(thead);
  table.appendChild(tbody);

  const root = document.getElementById('table')!;
  root.innerHTML = '';
  root.appendChild(table);
}
