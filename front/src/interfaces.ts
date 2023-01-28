export interface DownloadQuery {
    download_url: string;
}

export interface DbRow {
    id: string;
    url: string;
    file_name: string;
    file_size: string;
    start_time: string;
    end_time: string;
    status: string;
}
