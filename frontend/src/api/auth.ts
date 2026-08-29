import { apiRequest } from './client'

export type LoginRequest = {
  email: string
  password: string
}

export type RegisterRequest = {
  username: string
  email: string
  password: string
}

export type AuthResponse = {
  access_token: string
  refresh_token?: string | null
  user_id: string
  email: string
  username?: string | null
  display_name?: string | null
}

export type UserResponse = {
  id: string
  email: string
  username?: string | null
  display_name?: string | null
  is_system_admin?: boolean
}

export async function login(req: LoginRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>('/api/v1/auth/login', {
    method: 'POST',
    body: req,
    skipAuth: true,
  })
}

export async function register(req: RegisterRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>('/api/v1/auth/register', {
    method: 'POST',
    body: req,
    skipAuth: true,
  })
}

export async function getCurrentUser(): Promise<UserResponse> {
  return apiRequest<UserResponse>('/api/v1/users/me')
}

export async function logout(): Promise<void> {
  await apiRequest<void>('/api/v1/auth/logout', { method: 'POST' })
}
