export interface SoundConfig {
    sample_rate: number;
}

export interface GlobalState {
    active_project: string | null;
    sound_config: SoundConfig;
    resources: string[];
}
