import './style.css'
import { API_URL } from './constants'
import { DbRow, DownloadQuery } from './interfaces';

const urlInput = document.getElementById('url-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-button') as HTMLButtonElement;
const downloadResult = document.getElementById('download-result') as HTMLElement;
const table = document.getElementById('table') as HTMLTableElement;

downloadButton.onclick = () => {
    const req = new XMLHttpRequest();
    req.open('POST', API_URL + '/download');
    const data: DownloadQuery = { download_url: urlInput.value.trim() };
    req.onload = () => {
        downloadResult.style.display = 'inline-block';
        downloadResult.innerText = req.response;
    };
    req.send(JSON.stringify(data));
};

urlInput.oninput = () => {
    downloadResult.style.display = 'none';
}

function updateTable() {
    const req = new XMLHttpRequest();
    req.open('GET', API_URL + '/data');
    req.onload = () => {
        table.innerHTML = '<tr><th>ID</th><th>URL</th><th>File Name</th><th>File Size</th><th>Start Time</th><th>End Time</th><th>Status</th></tr>';
        const data: DbRow[] = JSON.parse(req.response);
        for (const row of data) {
            const tableRow = document.createElement('tr');
            for (const [_key, value] of Object.entries(row)) {
                const tableColumn = document.createElement('td');
                tableColumn.innerText = value;
                tableRow.appendChild(tableColumn);
            }
            table.appendChild(tableRow);
        }
    };
    req.send();
}

updateTable();
setInterval(updateTable, 5000);
