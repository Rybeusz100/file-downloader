import './style.css';
import './tabulator.min.css'
import { API_URL } from './constants';
import { DbRow, DownloadQuery } from './interfaces';
import { TabulatorFull as Tabulator } from 'tabulator-tables';

const urlInput = document.getElementById('url-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-button') as HTMLButtonElement;
const downloadResult = document.getElementById('download-result') as HTMLElement;

downloadButton.onclick = () => {
    const req = new XMLHttpRequest();
    req.open('POST', API_URL + '/download');
    req.setRequestHeader('Content-Type', 'application/json');
    const data: DownloadQuery = { download_url: urlInput.value.trim() };
    req.onload = () => {
        downloadResult.style.display = 'inline-block';
        downloadResult.innerText = req.response;
    };
    req.send(JSON.stringify(data));
};

urlInput.oninput = () => {
    downloadResult.style.display = 'none';
};

function updateTable() {
    const req = new XMLHttpRequest();
    req.open('GET', API_URL + '/data');
    req.onload = () => {
        const data: DbRow[] = JSON.parse(req.response);
        new Tabulator('#table', {
            data: data,
            autoColumns: true
        })
    };
    req.send();
}

updateTable();
setInterval(updateTable, 1000);
