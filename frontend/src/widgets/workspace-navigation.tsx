import { useEffect, useState } from 'react'
import { useLocation } from 'react-router'
import {
  ChevronRight,
  ClipboardList,
  FileCheck2,
  FileText,
  GitBranch,
  LayoutDashboard,
  Search,
} from 'lucide-react'
import { Button, NamespaceNavLink, ProjectNavigationGroup, SidebarItem } from '@sdlc/ui/ui'
import { projectIdentity, withNamespaceLocation, type useProjectDisclosures } from '@sdlc/ui/lib'
import { documentPath, useWorkspaceCatalog, workspacePath, type Workspace } from '@/api/workspaces'
import { useSpaceTree } from '@/shared/api/hooks'
import type { components } from '@/api/generated'
import { useNamespaceContext } from './namespace-context'
export const projectResourceLabel = (project: Workspace) =>
  project.resource_status === 'not_connected'
    ? 'Wiki не подключена'
    : project.resource_status === 'forbidden'
      ? 'Нет доступа к Wiki'
      : project.resource_status === 'unavailable'
        ? 'Привязка Wiki недоступна'
        : project.state === 'archived'
          ? 'В архиве'
          : project.stale
            ? 'Метаданные устарели'
            : undefined
export function useDocumentDisclosures() {
  const [values, setValues] = useState<Record<string, boolean>>(() => {
    try {
      const value: unknown = JSON.parse(localStorage.getItem('wiki:document-folds:v1') ?? '{}')
      return value && typeof value === 'object' && !Array.isArray(value)
        ? Object.fromEntries(Object.entries(value).filter(([, open]) => typeof open === 'boolean'))
        : {}
    } catch {
      return {}
    }
  })
  useEffect(() => {
    try {
      localStorage.setItem('wiki:document-folds:v1', JSON.stringify(values))
    } catch {
      /* Optional preferences. */
    }
  }, [values])
  return {
    values,
    toggle: (id: string, open: boolean) => setValues((current) => ({ ...current, [id]: !open })),
  }
}
type Documents = ReturnType<typeof useDocumentDisclosures>
type Node = components['schemas']['SpaceTreeNodeResponse']
function contains(nodes: Node[], id: string): boolean {
  return nodes.some((node) => node.id === id || contains(node.children, id))
}
function DocumentNodes({
  nodes,
  project,
  documents,
  prefix,
  current,
  all,
  search,
  onNavigate,
}: {
  nodes: Node[]
  project: Workspace
  documents: Documents
  prefix: string
  current: string
  all: boolean
  search: string
  onNavigate?: () => void
}) {
  return (
    <ul className="base-sidebar-list">
      {nodes.map((node) => {
        const key = prefix + '/' + node.id,
          open = documents.values[key] ?? contains(node.children, current)
        const to = withNamespaceLocation(documentPath(node.id, search, all), project.namespace)
        return (
          <li key={node.id}>
            <div className="flex min-w-0 items-center">
              {node.children.length > 0 && (
                <Button
                  size="icon"
                  variant="ghost"
                  aria-label={`Развернуть раздел: ${node.title}`}
                  aria-expanded={open}
                  onClick={() => documents.toggle(key, open)}
                >
                  <ChevronRight className={open ? 'rotate-90' : ''} aria-hidden />
                </Button>
              )}
              <SidebarItem asChild active={current === node.id}>
                <NamespaceNavLink to={to} title={node.title} onClick={onNavigate}>
                  <FileText aria-hidden />
                  <span className="base-sidebar-item-label truncate">{node.title}</span>
                </NamespaceNavLink>
              </SidebarItem>
            </div>
            {open && node.children.length > 0 && (
              <div className="ml-3">
                <DocumentNodes
                  {...{ project, documents, prefix, current, all, search, onNavigate }}
                  nodes={node.children}
                />
              </div>
            )}
          </li>
        )
      })}
    </ul>
  )
}
function DocumentTree({
  project,
  documents,
  open,
  all,
  onNavigate,
}: {
  project: Workspace
  documents: Documents
  open: boolean
  all: boolean
  onNavigate?: () => void
}) {
  const location = useLocation(),
    tree = useSpaceTree(project.space_key ?? '', open)
  const current = location.pathname.match(/^\/documents\/([^/]+)/)?.[1] ?? ''
  const prefix = project.binding
    ? `${project.binding.resource.instance_id}/${project.binding.resource.resource_id}`
    : ''
  if (!open) return null
  if (tree.isPending)
    return (
      <p role="status" className="px-3 text-sm">
        Загружаем документы…
      </p>
    )
  if (tree.isError)
    return (
      <div role="alert" className="px-3 text-sm">
        Дерево документов недоступно.{' '}
        <Button variant="ghost" onClick={() => tree.refetch()}>
          Повторить
        </Button>
      </div>
    )
  if (!tree.data.documents.length)
    return <p className="px-3 text-sm text-text-muted">Документов пока нет</p>
  return (
    <DocumentNodes
      nodes={tree.data.documents}
      {...{ project, documents, prefix, current, all, onNavigate }}
      search={location.search}
    />
  )
}
export function WorkspaceNavigation({
  disclosures,
  documents,
  compact = false,
  onNavigate,
}: {
  disclosures: ReturnType<typeof useProjectDisclosures>
  documents: Documents
  compact?: boolean
  onNavigate?: () => void
}) {
  const catalog = useWorkspaceCatalog(),
    context = useNamespaceContext(),
    location = useLocation()
  if (context.malformed || catalog.isError)
    return (
      <div role="alert" className="p-3 text-sm">
        Проекты недоступны.{' '}
        <Button variant="ghost" onClick={() => catalog.refetch()}>
          Повторить
        </Button>
      </div>
    )
  if (catalog.isPending)
    return (
      <p role="status" className="p-3 text-sm">
        Загружаем проекты…
      </p>
    )
  const projects = catalog.data.filter(
    (project) =>
      context.all ||
      (context.ref && projectIdentity(context.ref) === projectIdentity(project.namespace)),
  )
  const sections = [
    ['', 'Обзор', LayoutDashboard],
    ['documents', 'Документы', FileText],
    ['tasks', 'Задачи', ClipboardList],
    ['phases', 'Фазы', GitBranch],
    ['evidence', 'Материалы', FileCheck2],
    ['search', 'Поиск', Search],
  ] as const
  return (
    <nav aria-label="Проекты" className="base-sidebar-list mt-3 border-t border-border pt-3">
      {projects.map((project) => {
        const id = `${project.tracker.instance_id}/${project.tracker.resource_id}`
        const active =
          !!context.ref && projectIdentity(context.ref) === projectIdentity(project.namespace)
        const prefix = project.binding
          ? `${project.binding.resource.instance_id}/${project.binding.resource.resource_id}`
          : ''
        const treeKey = prefix + '/documents',
          treeOpen =
            documents.values[treeKey] ?? (active && location.pathname.includes('/documents'))
        return (
          <ProjectNavigationGroup
            key={id}
            name={project.name}
            projectKey={project.project_key}
            compact={compact}
            active={active}
            open={!disclosures.collapsed.includes(id)}
            onToggle={() => disclosures.toggle(id)}
            status={projectResourceLabel(project)}
          >
            {sections
              .filter(([section]) => !section || project.resource_status === 'connected')
              .map(([section, label, Icon]) => {
                const path = workspacePath(project.namespace, section),
                  to = withNamespaceLocation(
                    path + (context.all ? '?project_scope=all' : ''),
                    project.namespace,
                  )
                return (
                  <div key={section}>
                    <div className="flex min-w-0 items-center">
                      <SidebarItem asChild compact={compact} active={location.pathname === path}>
                        <NamespaceNavLink
                          to={to}
                          end
                          onClick={onNavigate}
                          title={label}
                          aria-label={label}
                        >
                          <Icon aria-hidden />
                          <span className="base-sidebar-item-label">{label}</span>
                        </NamespaceNavLink>
                      </SidebarItem>
                      {section === 'documents' && !compact && (
                        <Button
                          variant="ghost"
                          size="icon"
                          aria-label={`Дерево документов: ${project.name} · ${project.project_key}`}
                          aria-expanded={treeOpen}
                          onClick={() => documents.toggle(treeKey, treeOpen)}
                        >
                          <ChevronRight className={treeOpen ? 'rotate-90' : ''} aria-hidden />
                        </Button>
                      )}
                    </div>
                    {section === 'documents' && !compact && !disclosures.collapsed.includes(id) && (
                      <DocumentTree
                        project={project}
                        documents={documents}
                        open={treeOpen}
                        all={context.all}
                        onNavigate={onNavigate}
                      />
                    )}
                  </div>
                )
              })}
          </ProjectNavigationGroup>
        )
      })}
      {!projects.length && <p className="p-3 text-sm text-text-muted">Нет доступных проектов</p>}
    </nav>
  )
}
