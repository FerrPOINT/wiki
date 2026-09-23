import { useCallback } from 'react'
import { useBeforeUnload, useBlocker } from 'react-router'
import { ConfirmDialog } from '@sdlc/ui/ui'

interface UnsavedChangesGuardProps {
  when: boolean
}

export function UnsavedChangesGuard({ when }: UnsavedChangesGuardProps) {
  const blocker = useBlocker(when)

  useBeforeUnload(
    useCallback(
      (event) => {
        if (!when) return
        event.preventDefault()
        event.returnValue = ''
      },
      [when],
    ),
  )

  return (
    <ConfirmDialog
      open={blocker.state === 'blocked'}
      onOpenChange={(open) => {
        if (!open && blocker.state === 'blocked') blocker.reset()
      }}
      title="Покинуть страницу?"
      description="Несохранённые изменения будут потеряны. Сохраните черновик или подтвердите переход."
      onConfirm={() => {
        if (blocker.state === 'blocked') blocker.proceed()
      }}
    />
  )
}
