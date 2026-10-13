export function wikiSearchScope(search: string, hasProject: boolean) {
  const values = new URLSearchParams(search).getAll('search_scope')
  const scope = values[0] ?? (hasProject ? 'project' : 'all')
  return {
    scope: scope as 'project' | 'all',
    malformed:
      values.length > 1 ||
      !['project', 'all'].includes(scope) ||
      (scope === 'project' && !hasProject),
  }
}
