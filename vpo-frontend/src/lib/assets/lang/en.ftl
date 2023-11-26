socket =
    .attack = Attack
    .audio = Audio
    .decay = Decay
    .frequency = Frequency
    .gain = Gain
    .gate = Gate
    .midi = Midi
    .input-numbered = Input { $x }
    .release = Release
    .resonance = Resonance
    .speed = Speed
    .sustain = Sustain
    .value = Value
    .variable-numbered = x{ $x }
    .default = Default
    .transpose = Transpose
    .detune = Detune (cents)
    .db_gain = Gain (dB)
    .shelf_db_gain = 3rd+ harmonic gain (dB)
    .activate = Activate
    .load_mode = Set mode to "load"
    .set_mode = Set mode to "set"
    .map_set_mode = Set mode to "map set"
    .set_state = Set state
    .state = State
    .engage = Engage

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
    .MidiSwitchNode = Midi Switch
    .MidiToValueNode = Midi To Value
    .MidiToValuesNode = Midi To Values
    .MixerNode = Mixer
    .MemoryNode = Memory
    .OscillatorNode = Oscillator
    .OutputNode = Output
    .OutputsNode = Outputs
    .PolyphonicNode = Polyphonic
    .PortamentoNode = Portamento
    .RankPlayerNode = Rank Player
    .StreamExpressionNode = Expression (stream)
    .ToggleNode = Toggle
    .WavetableNode = Wavetable Oscillator
    .NoteMergerNode = Note Merger
    .MidiTransposeNode = Midi Transpose
    .WavetableSequencerNode = Wavetable Sequencer

property =
    .channels = Channels
    .input_count = Input count
    .polyphony = Polyphony
    .rank = Rank
    .expression = Expression
    .values_in_count = Values in count
    .wavetable = Wavetable
    .ui_name = UI name