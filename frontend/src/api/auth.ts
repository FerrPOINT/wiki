import { apiRequest } from './client'
import type {
  WikiAuthResponse,
  WikiLoginRequest,
  WikiRegisterRequest,
  WikiUserResponse,
} from './generated-exports'

export type LoginRequest = WikiLoginRequest

export type RegisterRequest = WikiRegisterRequest

export type AuthResponse = WikiAuthResponse

export type UserResponse = WikiUserResponse

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
