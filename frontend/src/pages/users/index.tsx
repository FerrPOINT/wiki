import { useEffect, useState } from 'react'
import { ExternalLink } from 'lucide-react'
import { useSearchParams } from 'react-router'
import {
  PageHeader,
  Button,
  EmptyState,
  ErrorState,
  LoadingState,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  usePlatformServices,
  ResourceList,
  ResourceListRow,
  SearchInput,
} from '@pdlc/ui/ui'
import { useUsers } from '@/shared/api/hooks'
import { formatApiErrorForUser } from '@/shared/lib/api-error'

const pageSize = 25
const searchDebounceMs = 300

function normalized(value: string | null): string {
  return value?.trim() ?? ''
}

function parsePage(value: string | null): number {
  if (!value || !/^[1-9]\d*$/.test(value)) return 1
  const parsed = Number(value)
  return Number.isSafeInteger(parsed) ? parsed : 1
}

function setOptionalParam(params: URLSearchParams, name: string, value: string): void {
  if (value) params.set(name, value)
  else params.delete(name)
}

function useUserDirectoryUrl() {
  const [searchParams, setSearchParams] = useSearchParams()
  const appliedSearch = normalized(searchParams.get('q'))
  const requestedPage = parsePage(searchParams.get('page'))
  const [search, setSearch] = useState(appliedSearch)

  useEffect(() => {
    setSearch(appliedSearch)
  }, [appliedSearch])

  useEffect(() => {
    const canonicalPage = requestedPage === 1 ? '' : String(requestedPage)
    const shouldCanonicalize =
      searchParams.getAll('q').length !== (appliedSearch ? 1 : 0) ||
      searchParams.get('q') !== (appliedSearch || null) ||
      searchParams.getAll('page').length !== (canonicalPage ? 1 : 0) ||
      searchParams.get('page') !== (canonicalPage || null)
    if (!shouldCanonicalize) return

    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        setOptionalParam(next, 'q', appliedSearch)
        setOptionalParam(next, 'page', canonicalPage)
        return next
      },
      { replace: true },
    )
  }, [appliedSearch, requestedPage, searchParams, setSearchParams])

  useEffect(() => {
    const nextSearch = search.trim()
    if (nextSearch === appliedSearch) return
    const timeout = window.setTimeout(() => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current)
          setOptionalParam(next, 'q', nextSearch)
          next.delete('page')
          return next
        },
        { replace: true },
      )
    }, searchDebounceMs)
    return () => window.clearTimeout(timeout)
  }, [appliedSearch, search, setSearchParams])

  function setPage(nextPage: number, replace = false) {
    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        setOptionalParam(next, 'page', nextPage > 1 ? String(nextPage) : '')
        return next
      },
      { replace },
    )
  }

  return { requestedPage, search, setPage, setSearch }
}

export function UsersPage() {
  const usersQuery = useUsers()
  const { services } = usePlatformServices()
  const { requestedPage, search, setPage, setSearch } = useUserDirectoryUrl()
  const adminUrl = services.find((service) => service.key === 'admin-panel')?.ui_url
  const users = usersQuery.data?.users ?? []
  const query = search.trim().toLocaleLowerCase('ru')
  const filtered = query
    ? users.filter((user) =>
        [user.display_name, user.username, user.email].some((value) =>
          value?.toLocaleLowerCase('ru').includes(query),
        ),
      )
    : users
  const totalPages = Math.max(1, Math.ceil(filtered.length / pageSize))
  const currentPage = Math.min(requestedPage, totalPages)
  const visibleUsers = filtered.slice((currentPage - 1) * pageSize, currentPage * pageSize)
  const firstVisible = filtered.length === 0 ? 0 : (currentPage - 1) * pageSize + 1
  const lastVisible = Math.min(currentPage * pageSize, filtered.length)

  useEffect(() => {
    if (usersQuery.isLoading || usersQuery.isError || requestedPage === currentPage) return
    setPage(currentPage, true)
  }, [currentPage, requestedPage, setPage, usersQuery.isError, usersQuery.isLoading])

  return (
    <div className="min-w-0 space-y-5">
      <PageHeader
        title={'Профили Wiki'}
        context={
          !usersQuery.isLoading &&
          !usersQuery.isError && (
            <p className="text-sm text-text-muted" aria-live="polite">
              {query ? `Найдено: ${filtered.length} из ${users.length}` : `Всего: ${users.length}`}
            </p>
          )
        }
        actions={
          adminUrl && (
            <Button asChild variant="secondary">
              <a href={`${adminUrl.replace(/\/$/, '')}/users`}>
                <ExternalLink className="h-4 w-4" aria-hidden />
                Учётные записи
              </a>
            </Button>
          )
        }
      />

      <div className="max-w-xl">
        <SearchInput
          aria-label="Поиск пользователей"
          placeholder="Имя или email"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
      </div>

      {usersQuery.isLoading && <LoadingState message="Загружаем пользователей" />}
      {usersQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(usersQuery.error, 'Не удалось загрузить пользователей')}
          onRetry={() => usersQuery.refetch()}
        />
      )}
      {!usersQuery.isLoading && !usersQuery.isError && filtered.length === 0 && (
        <EmptyState
          message={users.length === 0 ? 'Профилей пока нет' : 'По вашему запросу ничего не найдено'}
        />
      )}
      {!usersQuery.isLoading && !usersQuery.isError && visibleUsers.length > 0 && (
        <>
          <ResourceList asChild>
            <div className="md:hidden">
              {visibleUsers.map((user) => (
                <ResourceListRow asChild key={user.id}>
                  <div className="min-w-0 space-y-1">
                    <div className="flex items-start justify-between gap-3">
                      <span className="min-w-0 break-words font-medium">
                        {user.display_name ?? user.username ?? user.email}
                      </span>
                      <span className="shrink-0 text-xs text-text-secondary">
                        {user.active === false ? 'Профиль отключён' : 'Профиль доступен'}
                      </span>
                    </div>
                    <div className="break-all text-sm text-text-muted">{user.email}</div>
                  </div>
                </ResourceListRow>
              ))}
            </div>
          </ResourceList>
          <Table className="hidden table-fixed md:table">
            <TableHeader>
              <TableRow>
                <TableHead>Имя</TableHead>
                <TableHead>Email</TableHead>
                <TableHead className="w-44">Профиль Wiki</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {visibleUsers.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="break-words font-medium">
                    {user.display_name ?? user.username ?? user.email}
                  </TableCell>
                  <TableCell className="break-all">{user.email}</TableCell>
                  <TableCell>
                    {user.active === false ? 'Профиль отключён' : 'Профиль доступен'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
          {filtered.length > pageSize && (
            <nav
              aria-label="Страницы профилей"
              className="flex flex-wrap items-center justify-end gap-2"
            >
              <span className="mr-auto text-sm text-text-muted">
                Показаны {firstVisible}–{lastVisible} из {filtered.length}
              </span>
              <Button
                type="button"
                size="sm"
                variant="outline"

                disabled={currentPage === 1}
                onClick={() => setPage(currentPage - 1)}
              >
                Назад
              </Button>
              <span className="text-sm text-text-muted">
                {currentPage} / {totalPages}
              </span>
              <Button
                type="button"
                size="sm"
                variant="outline"

                disabled={currentPage === totalPages}
                onClick={() => setPage(currentPage + 1)}
              >
                Далее
              </Button>
            </nav>
          )}
        </>
      )}
    </div>
  )
}
