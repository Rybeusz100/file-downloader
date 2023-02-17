import './css/style.css';
import './css/tabulator.min.css';
import { API_URL } from './lib/constants';
import { DbRow, DownloadQuery } from './lib/interfaces';
import { TabulatorFull as Tabulator } from 'tabulator-tables';
import { formatBytes, login } from './lib/utils';

const urlInput = document.getElementById('url-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-button') as HTMLButtonElement;
const loginButton = document.getElementById('login-button') as HTMLButtonElement;
const downloadResult = document.getElementById('download-result') as HTMLElement;

let dataJson = '';

const table = new Tabulator('#table', {
    layout: 'fitColumns',
    columns: [
        { title: 'ID', field: 'id' },
        { title: 'URL', field: 'url' },
        { title: 'File Name', field: 'file_name' },
        { title: 'File Size', field: 'file_size' },
        { title: 'Start Time', field: 'start_time' },
        { title: 'End Time', field: 'end_time' },
        { title: 'Status', field: 'status' },
    ],
});

downloadButton.onclick = () => {
    downloadResult.style.display = 'inline-block';
    const token = localStorage.getItem('token');
    if (!token) {
        downloadResult.innerText = 'Login first';
        return;
    }
    const data: DownloadQuery = { download_url: urlInput.value.trim() };
    const options = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(data),
    };
    fetch(API_URL + '/restricted/download', options).then(async (response) => {
        downloadResult.innerText = await response.text();
    });
};

loginButton.onclick = () => {
    localStorage.removeItem('token');
    login();
    table.clearData();
};

urlInput.oninput = () => {
    downloadResult.style.display = 'none';
};

function updateTable() {
    const token = localStorage.getItem('token');
    if (!token) {
        setTimeout(updateTable, 1000);
        return;
    }
    const options = {
        method: 'GET',
        headers: {
            Authorization: `Bearer ${token}`,
        },
    };
    fetch(API_URL + '/restricted/data', options)
        .then((response) => {
            if (response.ok) {
                return response.text();
            } else {
                throw new Error('Request failed');
            }
        })
        .then((data) => {
            if (data !== dataJson) {
                dataJson = data;
                const newData: DbRow[] = JSON.parse(data).map((entry: DbRow) => {
                    return {
                        ...entry,
                        file_size: entry.file_size ? formatBytes(entry.file_size) : entry.file_size,
                    };
                });
                table.updateOrAddData(newData);
            }
        })
        .catch((error) => {
            console.error(error);
        })
        .finally(() => {
            setTimeout(updateTable, 1000);
        });
}

updateTable();
