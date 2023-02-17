import { API_URL } from './constants';

export function formatBytes(bytes: number, decimals = 2): string {
    if (bytes === 0) return '0 B';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

export function login() {
    const username = window.prompt('Enter your username:');
    const password = window.prompt('Enter your password:');
    const encodedCredentials = btoa(username + ':' + password);

    const options = {
        method: 'GET',
        headers: { Authorization: 'Basic ' + encodedCredentials },
    };

    fetch(API_URL + '/auth', options)
        .then((response) => {
            if (response.ok) {
                return response.json();
            } else if (response.status === 401) {
                throw new Error('Incorrect username or password');
            }
            throw new Error('Request failed');
        })
        .then((token) => {
            localStorage.setItem('token', token);
        })
        .catch((error) => {
            console.error(error);
        });
}
