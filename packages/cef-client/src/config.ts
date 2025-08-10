export interface Config {
    requestTimeout: number;
    defaultUrl: string,
    logging: boolean,
}

// Default configuration
const DEFAULT_CONFIG: Config = {
    requestTimeout: 50000,
    defaultUrl: 'about:blank',
    logging: true,
};

let globalConfig: Config = { ...DEFAULT_CONFIG };

export function getConfig(): Config {
    return globalConfig;
}

export function setConfig(config: Partial<Config>) {
    globalConfig = { ...globalConfig, ...config };
}