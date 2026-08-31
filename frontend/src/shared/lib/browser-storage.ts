export type SafeBrowserStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>

const memoryStorage = (() => {
  const values = new Map<string, string>()
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value)
    },
    removeItem: (key: string) => {
      values.delete(key)
    },
  } satisfies SafeBrowserStorage
})()

function isStorageLike(storage: unknown): storage is SafeBrowserStorage {
  return (
    typeof storage === 'object' &&
    storage !== null &&
    'getItem' in storage &&
    'setItem' in storage &&
    'removeItem' in storage &&
    typeof storage.getItem === 'function' &&
    typeof storage.setItem === 'function' &&
    typeof storage.removeItem === 'function'
  )
}

export function getSafeBrowserStorage(): SafeBrowserStorage {
  try {
    const storage = typeof window !== 'undefined' ? window.localStorage : undefined
    if (isStorageLike(storage)) return storage
  } catch {
    // Browser storage can be unavailable in private/opaque/test contexts.
  }
  return memoryStorage
}
