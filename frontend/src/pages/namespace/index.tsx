import { ErrorState } from '@pdlc/ui/ui'
import { Navigate, useLocation } from 'react-router'
import { withNamespaceLocation } from '@pdlc/ui/lib'
import { workspacePath } from '@/api/workspaces'
import { useNamespaceContext } from '@/widgets/namespace-context'
export function NamespacePage() {
  const { ref, malformed } = useNamespaceContext(),
    location = useLocation()
  if (malformed) return <ErrorState message="Некорректный контекст проекта." />
  return (
    <Navigate
      replace
      to={
        ref
          ? withNamespaceLocation(workspacePath(ref) + location.search + location.hash, ref)
          : '/projects?project_scope=all'
      }
    />
  )
}
