import { Link, useLocation } from 'react-router'
import { MoreHorizontal } from 'lucide-react'
import { withNamespaceLocation, type NamespaceLocation } from '@pdlc/ui/lib'
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@pdlc/ui/ui'
import { workspacePath } from '@/api/workspaces'

export function DossierMenu({
  spaceKey,
  namespace,
}: {
  spaceKey: string
  namespace?: NamespaceLocation | null
}) {
  const { search } = useLocation()
  const params = new URLSearchParams({ space: spaceKey })
  if (new URLSearchParams(search).get('project_scope') === 'all') params.set('project_scope', 'all')
  const target = (section: string) => {
    const path = (namespace ? workspacePath(namespace, section) : `/${section}`) + '?' + params
    return namespace ? withNamespaceLocation(path, namespace) : path
  }
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" aria-label={`Дополнительно: ${spaceKey}`}>
          <MoreHorizontal className="mr-2 h-4 w-4" aria-hidden />
          Дополнительно
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem asChild>
          <Link to={target('tasks')}>Документы по задачам</Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild>
          <Link to={target('phases')}>Документы по фазам</Link>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
