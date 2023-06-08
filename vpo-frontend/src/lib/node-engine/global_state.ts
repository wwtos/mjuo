export interface SoundConfig {
    sampleRate: number;
}

export interface GlobalState {
    activeProject: string | null;
    soundConfig: SoundConfig;
    resources: {
        ui: { [key: string]: any }
    };
}
