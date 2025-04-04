import type { noop } from './types.js'
import { View } from './types.ts'
import Select from './Select.js'
import Separator, { VerticalSeparator } from './Separator.js'
import { Title } from './Help.tsx'

const transitionTimes = [32, 24, 16, 12, 8, 6, 4, 3, 2, 1.5, 1, 0.75, 5, 0.25]
type TransitionTime = (typeof transitionTimes)[number]

type HeaderProps = {
  bpm: number
  fps: number
  isEncoding: boolean
  isQueued: boolean
  isRecording: boolean
  paused: boolean
  perfMode: boolean
  sketchName: string
  sketchNames: string[]
  tapTempoEnabled: boolean
  transitionTime: TransitionTime
  view: View
  onAdvance: noop
  onCaptureFrame: noop
  onChangePerfMode: noop
  onChangeTapTempoEnabled: noop
  onChangeTransitionTime: (transitionTime: TransitionTime) => void
  onChangeView: noop
  onClearBuffer: noop
  onClickRandomize: noop
  onQueueRecord: noop
  onRecord: noop
  onReset: noop
  onSave: noop
  onSwitchSketch: (sketchName: string) => void
  onTogglePlay: noop
}

export default function Header({
  bpm,
  fps,
  isEncoding,
  isQueued,
  isRecording,
  paused,
  perfMode,
  sketchName,
  sketchNames,
  tapTempoEnabled,
  transitionTime,
  view,
  onAdvance,
  onCaptureFrame,
  onChangePerfMode,
  onChangeTapTempoEnabled,
  onChangeTransitionTime,
  onChangeView,
  onClearBuffer,
  onClickRandomize,
  onQueueRecord,
  onRecord,
  onReset,
  onSave,
  onSwitchSketch,
  onTogglePlay,
}: HeaderProps) {
  return (
    <header>
      <section>
        <button onClick={onCaptureFrame}>Image</button>
        <VerticalSeparator />
        <button onClick={onTogglePlay}>{paused ? 'Play' : 'Pause'}</button>
        <button disabled={!paused} onClick={onAdvance}>
          Advance
        </button>
        <button onClick={onReset}>Reset</button>
        <VerticalSeparator />
        <button onClick={onClearBuffer}>Clear Buf.</button>
        <VerticalSeparator />
        <button
          className={isQueued ? 'on' : ''}
          disabled={isRecording || isEncoding}
          onClick={onQueueRecord}
        >
          {isQueued ? 'QUEUED' : 'Q Rec.'}
        </button>
        <button
          className={isRecording ? 'record-button on' : 'record-button'}
          disabled={isEncoding}
          onClick={onRecord}
        >
          {isRecording ? 'STOP' : isEncoding ? 'Encoding' : 'Rec.'}
        </button>
        <VerticalSeparator />
        <div className="meter">
          FPS: <span className="meter-value">{fps.toFixed(1)}</span>
        </div>
      </section>
      <Separator style={{ margin: '2px 0' }} />
      <section>
        <Select
          title={Title.Sketch}
          value={sketchName}
          options={sketchNames}
          onChange={onSwitchSketch}
          style={{ width: '128px' }}
        />
        <fieldset>
          <input
            title={Title.Perf}
            id="perf"
            type="checkbox"
            checked={perfMode}
            onChange={onChangePerfMode}
          />
          <label htmlFor="perf">Perf.</label>
        </fieldset>
        <VerticalSeparator />
        <div className="meter">
          BPM: <span className="meter-value">{bpm.toFixed(1)}</span>
        </div>
        <fieldset>
          <input
            title={Title.Tap}
            id="tap"
            type="checkbox"
            checked={tapTempoEnabled}
            onChange={onChangeTapTempoEnabled}
          />
          <label htmlFor="tap">Tap</label>
        </fieldset>
        <VerticalSeparator />
        <Select
          title={Title.TransitionTime}
          style={{ width: '48px' }}
          value={transitionTime.toString()}
          options={transitionTimes}
          onChange={(value) => {
            onChangeTransitionTime(parseFloat(value))
          }}
        />
        <button title={Title.Random} onClick={onClickRandomize}>
          ?
        </button>
        <VerticalSeparator />
        <button title={Title.Save} onClick={onSave}>
          Save
        </button>
        <button
          title={Title.Settings}
          className={view === View.Settings ? 'on' : ''}
          onClick={onChangeView}
        >
          Conf.
        </button>
      </section>
    </header>
  )
}
