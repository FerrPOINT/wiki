import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router'

const previousCursorParam = 'previous_cursor'

function normalized(value: string | null): string {
  return value?.trim() ?? ''
}

function setOptionalParam(params: URLSearchParams, name: string, value: string): void {
  if (value) params.set(name, value)
  else params.delete(name)
}

export function clearDossierCatalogParams(params: URLSearchParams): void {
  params.delete('q')
  params.delete('cursor')
  params.delete(previousCursorParam)
}

export function pathWithSearchParams(path: string, params: URLSearchParams): string {
  const query = params.toString()
  return query ? `${path}?${query}` : path
}

export function useDossierCatalogUrl() {
  const [searchParams, setSearchParams] = useSearchParams()
  const appliedSearch = normalized(searchParams.get('q'))
  const cursor = normalized(searchParams.get('cursor')) || undefined
  const previousCursors = useMemo(
    () =>
      searchParams
        .getAll(previousCursorParam)
        .map((value) => value.trim())
        .filter(Boolean),
    [searchParams],
  )
  const [search, setSearch] = useState(appliedSearch)

  useEffect(() => {
    setSearch(appliedSearch)
  }, [appliedSearch])

  useEffect(() => {
    const rawQuery = searchParams.get('q')
    const rawCursor = searchParams.get('cursor')
    const rawPrevious = searchParams.getAll(previousCursorParam)
    const canonicalPrevious = cursor ? previousCursors : []
    const shouldCanonicalize =
      rawQuery !== (appliedSearch || null) ||
      rawCursor !== (cursor || null) ||
      rawPrevious.length !== canonicalPrevious.length ||
      rawPrevious.some((value, index) => value !== canonicalPrevious[index])
    if (!shouldCanonicalize) return

    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        setOptionalParam(next, 'q', appliedSearch)
        setOptionalParam(next, 'cursor', cursor ?? '')
        next.delete(previousCursorParam)
        for (const previousCursor of canonicalPrevious) {
          next.append(previousCursorParam, previousCursor)
        }
        return next
      },
      { replace: true },
    )
  }, [appliedSearch, cursor, previousCursors, searchParams, setSearchParams])

  useEffect(() => {
    const normalizedSearch = search.trim()
    if (normalizedSearch === appliedSearch) return
    const timeout = window.setTimeout(() => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current)
          setOptionalParam(next, 'q', normalizedSearch)
          next.delete('cursor')
          next.delete(previousCursorParam)
          return next
        },
        { replace: true },
      )
    }, 300)
    return () => window.clearTimeout(timeout)
  }, [search, appliedSearch, setSearchParams])

  function changePage(direction: 'previous' | 'next', nextCursor?: string) {
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      const currentCursor = normalized(next.get('cursor'))
      const history = next
        .getAll(previousCursorParam)
        .map((value) => value.trim())
        .filter(Boolean)

      next.delete(previousCursorParam)
      if (direction === 'next') {
        const normalizedNextCursor = nextCursor?.trim() ?? ''
        if (!normalizedNextCursor) return current
        if (currentCursor) history.push(currentCursor)
        next.set('cursor', normalizedNextCursor)
      } else {
        const previousCursor = history.pop()
        setOptionalParam(next, 'cursor', previousCursor ?? '')
      }
      for (const previousCursor of history) next.append(previousCursorParam, previousCursor)
      return next
    })
  }

  return {
    appliedSearch,
    changePage,
    currentPage: cursor ? previousCursors.length + 2 : 1,
    cursor,
    hasPreviousPage: Boolean(cursor),
    search,
    searchParams,
    setSearch,
  }
}
