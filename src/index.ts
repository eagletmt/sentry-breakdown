import Papa from 'papaparse';
import Plotly from 'plotly.js-basic-dist';

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

interface RowProps {
  'Date': string;
  'Project Slug': string;
  'Total Errors': string;
}

class Row {
  public readonly date: string;
  public readonly projectSlug: string;
  public readonly totalErrors: number;

  constructor(props: RowProps) {
    this.date = props['Date'];
    this.projectSlug = props['Project Slug'];
    this.totalErrors = parseInt(props['Total Errors'], 10);
  }
}

function onCsvParseComplete(results: Papa.ParseResult<RowProps>) {
  if (results.data.length === 0) {
    return;
  }
  const header = document.createElement('tr');
  for (const key of ['Date', 'Project slug', 'Total errors']) {
    const th = document.createElement('th');
    th.appendChild(document.createTextNode(key));
    header.appendChild(th);
  }
  const thead = document.createElement('thead');
  thead.appendChild(header);

  const tbody = document.createElement('tbody');
  const traces: { [key: string]: Partial<Plotly.ScatterData> } = {};
  for (const rawRow of results.data) {
    const row = new Row(rawRow);
    const tr = document.createElement('tr');
    const date = document.createElement('td');
    date.appendChild(document.createTextNode(row.date));
    const projectSlug = document.createElement('td');
    projectSlug.appendChild(document.createTextNode(row.projectSlug));
    const totalErrors = document.createElement('td');
    totalErrors.appendChild(document.createTextNode(row.totalErrors.toString()));
    tr.appendChild(date);
    tr.appendChild(projectSlug);
    tr.appendChild(totalErrors);
    tbody.appendChild(tr);

    let trace = traces[row.projectSlug];
    if (trace == null) {
      trace = {
        type: 'scatter',
        name: row.projectSlug,
        mode: 'lines+markers',
        x: [],
        y: [],
      };
      traces[row.projectSlug] = trace;
    }
    const x = trace.x as Plotly.Datum[];
    const y = trace.y as Plotly.Datum[];
    x.push(row.date);
    y.push(row.totalErrors);
  }

  const table = document.createElement('table');
  table.appendChild(thead);
  table.appendChild(tbody);

  const root = document.getElementById('table')!;
  root.innerHTML = '';
  root.appendChild(table);

  const chart = document.getElementById('chart')!;
  const width = document.documentElement.clientWidth;
  const height = width/16*9;
  chart.style.width = `${width}px`;
  chart.style.height = `${height}px`;
  Plotly.newPlot(chart, Object.values(traces));
}
