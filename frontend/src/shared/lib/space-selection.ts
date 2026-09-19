export const defaultSpaceKey = 'BASE'

export function resolveSpaceKey(
  requested: string | null,
  spaces: ReadonlyArray<{ key: string }>,
): string {
  const explicit = requested?.trim().toUpperCase()
  if (explicit) return explicit
  return (
    spaces.find((space) => space.key.toUpperCase() === defaultSpaceKey)?.key ?? spaces[0]?.key ?? ''
  )
}
