export interface DownloadQuery {
    download_url: string;
}

export interface DbRow {
    id: number;
    url: string;
    file_name: string;
    file_size: number;
    start_time: string;
    end_time: string;
    status: string;
    user_id: number;
}
