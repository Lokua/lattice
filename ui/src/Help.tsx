import { isMac } from './util'

const mod = isMac ? 'Cmd' : 'Ctrl'

export const Help = {
  Advance: format(
    `When the [Play/Pause] toggle is set to [Pause], allows manually advancing 
    frames (Shortcut: [${mod} A])`
  ),
  Audio: 'The Audio input device used for audio controls',
  Clear: format(
    `Clear any alpha blending or "fade trails" from frame persistence. Requires 
    your sketch is using the clear_color attribute via sketch_components macro`
  ),
  DeleteMappings: 'Delete all MIDI Mappings',
  DisableMappings: 'Disable/Enable MIDI Mappings',
  Exclusions: format(
    `Exclusions: select controls to exclude from Randomization (Shortcut: E)`
  ),
  Fps: 'The effective framerate over a 1 second running average',
  Hrcc: format(`
    Enable high resolution (14bit) MIDI for CCs 0-31 (requires support 
    from your MIDI device)
  `),
  Image: `Capture PNG to disk (Shortcut: [${mod} I])`,
  Mappings: format(`
    Mappings: allows mapping of external MIDI CCs to UI sliders, aka
    "MIDI Learn". Mappings are saved with the sketch when you click [Save]. 
  `),
  MidiClockPort:
    "The MIDI port used to sync all Lattice's frame counter and animations",
  MidiInputPort:
    'The MIDI port Lattice will listen to for incoming MIDI CC messages',
  MidiOutputPort: format(`
    The MIDI port Lattice will send internally stored MIDI values 
    to (use for resyncing controllers after changing sketches)
  `),
  OscPort: 'The OSC port Lattice will use for OSC controls',
  Play: format(`
    Play/Pause Toggle (Shortcut: [P]). When Pause is engaged, use the [Advance]
    button or [${mod} A] to manually advance frames.
  `),
  Perf: format(
    `Enable/disable Performance Mode. When enabled, prevents Lattice from 
    applying a sketch's default width and height and also disables automatic 
    window repositioning. This is necessary in live performance contexts where 
    you likely will fullsize the screen and want to keep it that way when 
    switching sketches`
  ),
  Queue: 'Queue recording to start upon receiving a MIDI Start message',
  Random: `Randomize all UI controls (Shortcut: [${mod} R])`,
  Reload: format(
    `Reload the current sketch back to its last saved state 
    (Shortcut: [Shift ${mod} R])`
  ),
  Record: 'Start/Stop recording',
  Reset: 'Reset the frame counter and all animations (Shortcut: [R])',
  Tap: `
    Enabled/disable tap tempo. When enabled, use the [Space] key to tap. 
    Note that keeping enabled will preserve the currently tapped-in tempo when 
    switching sketches; disabling will always revert to a sketch's configured BPM.
  `,
  TransitionTime: 'Snapshot and Randomization transition time (in beats)',
  Send: 'Sends the state of all CCs to the MIDI output port',
  Settings: 'Global settings and MIDI mappings',
  Sketch: 'Sketch chooser',
  Snapshots: format(`
    Snapshot Editor: store and recall up to 10 snapshots (Shortcut: [S]).
    You can also save snapshots via [Shift Digit] and recall them
    via [${mod} Digit] without entering the editor.
  `),
  Save: format(`
    Save UI control states and MIDI mappings for this sketch to disk 
    (Shortcut: [${mod} S])
  `),
}

function format(s: string): string {
  return s
    .split('\n')
    .filter((s) => s)
    .map((line) => line.trim())
    .join(' ')
}
