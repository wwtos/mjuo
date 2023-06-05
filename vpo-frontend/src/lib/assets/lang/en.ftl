socket =
    .attack = Attack
    .audio = Audio
    .decay = Decay
    .frequency = Frequency
    .gain = Gain
    .gate = Gate
    .midi = Midi
    .input-numbered = Input { $id }
    .release = Release
    .resonance = Resonance
    .speed = Speed
    .sustain = Sustain
    .value = Value
    .variable-numbered = x{ $id }
    .default = Default
    .transpose = Transpose
    .detune = Detune (cents)
    .air_amplitude = Gain (dB)
    .shelf_db_gain = 3rd+ harmonic gain (dB)
    .activate = Activate
    .load_mode = Set mode to "load"
    .set_mode = Set mode to "set"
    .map_set_mode = Set mode to "map set"
    .set_state = Set state
    .state = State

node =
    .BiquadFilterNode = Filter (12db/octave)
    .DummyNode = Dummy
    .EnvelopeNode = Envelope
    .ExpressionNode = Expression
    .FunctionNode = Function
    .GainNode = Gain
    .InputsNode = Inputs
    .MidiFilterNode = Midi Filter
    .MidiInNode = Midi In
    .MidiToValuesNode = Midi To Values
    .MixerNode = Mixer
    .MemoryNode = Memory
    .OscillatorNode = Oscillator
    .OutputNode = Output
    .OutputsNode = Outputs
    .PolyphonicNode = Polyphonic
    .PortamentoNode = Portamento
    .RankPlayerNode = Rank Player
    .StreamExpressionNode = Expression (stream)\
    .ToggleNode = Toggle
    .WavetableNode = Wavetable Oscillator
    .NoteMergerNode = Note Merger
    .MidiTransposeNode = Midi Transpose
    .WavetableSequencerNode = Wavetable Sequencer

property =
    .input_count = Input count
    .polyphony = Polyphony
    .rank = Rank
    .expression = Expression
    .values_in_count = Values in count
    .wavetable = Wavetable
    .ui_name = UI name