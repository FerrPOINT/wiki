import { FormField, Select } from '@pdlc/ui/ui'
import { useSearchParams } from 'react-router'
import { useSpaces } from '@/shared/api/hooks'
import { clearDossierCatalogParams } from '@/shared/lib/dossier-catalog-url'
import { resolveSpaceKey } from '@/shared/lib/space-selection'
import { useProjectRouteRef } from './namespace-context'

// Space resolution, URL normalization and namespace authority are Wiki rules.
export function useSelectedSpaceKey() {
  const [searchParams, setSearchParams] = useSearchParams()
  const spacesQuery = useSpaces()
  const selectedSpaceKey = resolveSpaceKey(
    searchParams.get('space'),
    spacesQuery.data?.spaces ?? [],
  )

  function setSelectedSpaceKey(spaceKey: string) {
    const normalized = spaceKey.trim().toUpperCase()
    setSearchParams((current) => {
      const nextParams = new URLSearchParams(current)
      if (normalized) nextParams.set('space', normalized)
      else nextParams.delete('space')
      clearDossierCatalogParams(nextParams)
      return nextParams
    })
  }

  return [selectedSpaceKey, setSelectedSpaceKey, spacesQuery, searchParams] as const
}

export function SpaceSelector({
  id,
  value,
  onChange,
  disabled = false,
}: {
  id: string
  value: string
  onChange: (spaceKey: string) => void
  disabled?: boolean
}) {
  const namespace = useProjectRouteRef()
  const spacesQuery = useSpaces()
  const spaces = spacesQuery.data?.spaces ?? []
  const hasSelected = spaces.some((space) => space.key === value)
  const options = value && !hasSelected ? [{ key: value, name: value }, ...spaces] : spaces
  return (
    <div className="w-full sm:w-64">
      <FormField id={id} label="Пространство">
        {(attrs) => (
          <Select
            {...attrs}
            value={value}
            onValueChange={onChange}
            disabled={
              disabled ||
              Boolean(namespace.ref) ||
              spacesQuery.isLoading ||
              spacesQuery.isError ||
              options.length === 0
            }
            options={
              options.length === 0
                ? [{ value: '', label: 'Нет пространств' }]
                : options.map((space) => ({
                    value: space.key,
                    label: space.name ? `${space.key} · ${space.name}` : space.key,
                  }))
            }
          />
        )}
      </FormField>
    </div>
  )
}
