import { useSpaces } from '@/shared/api/hooks'
import { useWorkspaceCatalog } from '@/api/workspaces'

/** Project lifecycle limits working choices without changing native space records. */
export function useActiveSpaces(includeArchived = false) {
  const spaces = useSpaces()
  const enabled = import.meta.env.VITE_NAMESPACE_ENABLED === 'true' && !includeArchived
  const catalog = useWorkspaceCatalog(enabled, true)
  const archived = new Set<string>()
  const active = new Set<string>()
  for (const project of catalog.data ?? []) {
    const target = project.state === 'archived' ? archived : active
    for (const space of project.spaces) target.add(space.id)
  }
  return {
    ...spaces,
    unfilteredCount: spaces.data?.spaces.length ?? 0,
    data:
      enabled && !catalog.isSuccess
        ? undefined
        : spaces.data && {
            ...spaces.data,
            spaces: spaces.data.spaces.filter(
              (space) =>
                includeArchived ||
                (space.status !== 'archived' && (!archived.has(space.id) || active.has(space.id))),
            ),
          },
    isLoading: spaces.isLoading || (enabled && catalog.isPending),
    isPending: spaces.isPending || (enabled && catalog.isPending),
    isError: spaces.isError || (enabled && catalog.isError),
    error: spaces.error ?? (enabled ? catalog.error : null),
    refetch: async () => {
      const [result] = await Promise.all([
        spaces.refetch(),
        enabled ? catalog.refetch() : undefined,
      ])
      return result
    },
  }
}
