export interface Config {
    requestTimeout: number;
    defaultUrl: string,
    logging: boolean,
}

const DEFAULT_CONFIG: Config = {
    requestTimeout: 50000,
    defaultUrl: 'huly://newtab',
    logging: false,
};

let globalConfig: Config = { ...DEFAULT_CONFIG };

export function getConfig(): Config {
    return globalConfig;
}

export function setConfig(config: Partial<Config>) {
    globalConfig = { ...globalConfig, ...config };
}