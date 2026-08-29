import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router'
import { getCurrentUser, login, logout, register } from '@/api/auth'
import { storeRefreshToken, useAuthStore } from '@/shared/auth/store'

const authKeys = {
  me: ['me'] as const,
}

export function useLogin() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: login,
    onSuccess: (data) => {
      storeRefreshToken(data.refresh_token ?? null)
      setAuth({
        token: data.access_token,
        userId: data.user_id,
        email: data.email,
        username: data.username ?? undefined,
        displayName: data.display_name ?? undefined,
      })
      queryClient.invalidateQueries({ queryKey: authKeys.me })
    },
  })
}

export function useRegister() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: register,
    onSuccess: (data) => {
      storeRefreshToken(data.refresh_token ?? null)
      setAuth({
        token: data.access_token,
        userId: data.user_id,
        email: data.email,
        username: data.username ?? undefined,
        displayName: data.display_name ?? undefined,
      })
      queryClient.invalidateQueries({ queryKey: authKeys.me })
    },
  })
}

export function useCurrentUser() {
  const token = useAuthStore((state) => state.token)
  return useQuery({
    queryKey: authKeys.me,
    queryFn: getCurrentUser,
    enabled: Boolean(token),
    staleTime: 5 * 60 * 1000,
  })
}

export function useLogout() {
  const clearAuth = useAuthStore((state) => state.logout)
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  return useMutation({
    mutationFn: logout,
    onSettled: () => {
      clearAuth()
      queryClient.clear()
      navigate('/login')
    },
  })
}
