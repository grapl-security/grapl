export type redirect = (pageName: string) => void;

export type PluginPayload = {
    plugins: object;
};

export type onRead = (fileResult: string) => void;

export type Event = {
    filename: string;
};

export type UploadFormState = {
    curFiles: FileList | null;
    success: boolean | null;
};

export type DirectoryUpload = {
    files: FileList;
};

export type MessageProps = {
    status: boolean | null;
};

export type PluginTableState = {
    rows: string[];
    toggle: boolean;
};
