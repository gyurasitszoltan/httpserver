import { computed, ref } from 'vue'
import { api, type CurrentUser } from './api'

const user = ref<CurrentUser | null>(null)
const loaded = ref(false)

export const auth = {
  user,
  loaded,
  isAdmin: computed(() => user.value?.role === 'admin'),
  async load() {
    try { user.value = await api.me() } catch { user.value = null } finally { loaded.value = true }
  },
  async logout() {
    await api.logout()
    user.value = null
  },
}
