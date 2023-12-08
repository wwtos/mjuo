export interface SoundConfig {
    sampleRate: number;
}

export interface GlobalState {
    activeProject: string | null;
    soundConfig: SoundConfig;

}

export interface Resources {
    ui: { [key: string]: any };
}
