import { UnsavedChangesGuard as SharedGuard } from '@sdlc/ui/ui'

interface UnsavedChangesGuardProps {
  when: boolean
}

export function UnsavedChangesGuard({ when }: UnsavedChangesGuardProps) {
  return (
    <SharedGuard when={when}
      title="Покинуть страницу?"
      description="Несохранённые изменения будут потеряны. Сохраните черновик или подтвердите переход."
    />
  )
}
