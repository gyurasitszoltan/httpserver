export type CurrentUser = { id: number; email: string; displayName: string | null; role: 'admin' | 'user' }
export type User = CurrentUser & { isActive: boolean; createdAt: number; lastLoginAt: number | null }
export type UsersPage = { items: User[]; page: number; pageSize: number; total: number }

type ErrorPayload = { error?: { message?: string } }

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...init.headers },
    ...init,
  })
  if (!response.ok) {
    const payload = (await response.json().catch(() => ({}))) as ErrorPayload
    throw new Error(payload.error?.message ?? 'A kérés sikertelen.')
  }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

export const api = {
  me: () => request<CurrentUser>('/api/auth/me'),
  requestMagicLink: (email: string) => request<void>('/api/auth/magic-link', { method: 'POST', body: JSON.stringify({ email }) }),
  verifyMagicLink: (token: string) => request<CurrentUser>('/api/auth/verify-magic-link', { method: 'POST', body: JSON.stringify({ token }) }),
  logout: () => request<void>('/api/auth/logout', { method: 'POST' }),
  users: (page = 1, query = '') => request<UsersPage>(`/api/admin/users?page=${page}&pageSize=25&query=${encodeURIComponent(query)}`),
  createUser: (user: { email: string; displayName: string | null; role: string; isActive: boolean }) => request<User>('/api/admin/users', { method: 'POST', body: JSON.stringify(user) }),
  updateUser: (id: number, user: Partial<{ displayName: string | null; role: string; isActive: boolean }>) => request<User>(`/api/admin/users/${id}`, { method: 'PATCH', body: JSON.stringify(user) }),
  deactivateUser: (id: number) => request<void>(`/api/admin/users/${id}`, { method: 'DELETE' }),
}
