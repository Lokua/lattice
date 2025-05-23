import React, { useEffect, useState } from 'react'
import { Mappings } from './types'
import IconButton from './IconButton'
import clsx from 'clsx/lite'

type Props = {
  sliderNames: string[]
  mappings: Mappings
  mappingsEnabled: boolean
  onChangeMappingsEnabled: () => void
  onDeleteMappings: () => void
  onRemoveMapping: (name: string) => void
  onSetCurrentlyMapping: (name: string) => void
}

export default function MapMode({
  sliderNames,
  mappings,
  mappingsEnabled,
  onChangeMappingsEnabled,
  onDeleteMappings,
  onRemoveMapping,
  onSetCurrentlyMapping,
}: Props) {
  const [currentlyMapping, setCurrentlyMapping] = useState('')

  useEffect(() => {
    document.addEventListener('click', onOutsideClick)
    document.addEventListener('keydown', onKeyDown)

    return () => {
      document.removeEventListener('click', onOutsideClick)
      document.removeEventListener('keydown', onKeyDown)
    }

    function onOutsideClick(e: MouseEvent) {
      if (
        currentlyMapping &&
        !(e.target as HTMLButtonElement)?.classList?.contains('map-button')
      ) {
        clearCurrentlyMapping()
      }
    }

    function onKeyDown(e: KeyboardEvent) {
      if (e.code === 'Enter') {
        clearCurrentlyMapping()
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentlyMapping])

  function clearCurrentlyMapping() {
    setCurrentlyMapping('')
    onSetCurrentlyMapping('')
  }

  function onClickMap(name: string) {
    if (currentlyMapping !== name) {
      setCurrentlyMapping(name)
      onSetCurrentlyMapping(name)
    }
  }

  return (
    <div id="map-mode">
      <header>
        <h2 data-help-id="Mappings">MIDI Mappings</h2>
        <section>
          <IconButton
            name="DisableMappings"
            data-help-id="DisableMappings"
            isToggle
            on={!mappingsEnabled}
            onClick={onChangeMappingsEnabled}
          />
          <IconButton
            name="DeleteMappings"
            data-help-id="DeleteMappings"
            onClick={onDeleteMappings}
          />
        </section>
      </header>
      <main>
        {sliderNames.map((name) => {
          const mapping = mappings[name]
          const isMapped = !!mapping
          const isMapping = currentlyMapping === name
          const text =
            !isMapping && !isMapped
              ? '—'
              : isMapping && !isMapped
              ? '...'
              : mapping.join('/')

          return (
            <React.Fragment key={name}>
              <label
                style={{
                  textDecoration: mappingsEnabled ? 'none' : 'line-through',
                }}
              >
                {isMapped ? <b>{name}</b> : name}
              </label>
              <span style={{ display: 'inline-flex' }}>
                <button
                  className={clsx(
                    'map-button',
                    isMapping && 'mapping',
                    !isMapping && !isMapped && 'inactive'
                  )}
                  disabled={!mappingsEnabled}
                  onClick={() => {
                    onClickMap(name)
                  }}
                >
                  {text}
                </button>
                {isMapped && (
                  <IconButton
                    name="Close"
                    onClick={() => {
                      onRemoveMapping(name)
                      clearCurrentlyMapping()
                    }}
                  />
                )}
              </span>
            </React.Fragment>
          )
        })}
      </main>
    </div>
  )
}
