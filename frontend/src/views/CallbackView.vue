<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '../api'
import { auth } from '../auth'

const route = useRoute()
const router = useRouter()
const error = ref('')

onMounted(async () => {
  const token = typeof route.query.token === 'string' ? route.query.token : ''
  if (!token) { error.value = 'Hiányzó belépési token.'; return }
  try {
    auth.user.value = await api.verifyMagicLink(token)
    auth.loaded.value = true
    await router.replace('/')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'A belépési link érvénytelen.'
    await router.replace('/auth/callback')
  }
})
</script>

<template>
  <section class="card mx-auto max-w-md overflow-hidden">
    <div class="h-1.5 bg-gradient-to-r from-brand-500 via-violet-500 to-fuchsia-400"></div>
    <div class="p-8 text-center">
      <template v-if="!error">
        <svg class="mx-auto h-8 w-8 animate-spin text-brand-500" viewBox="0 0 24 24" fill="none"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 0 1 8-8v4a4 4 0 0 0-4 4H4z"/></svg>
        <p class="mt-4 text-slate-600">Belépés ellenőrzése…</p>
      </template>
      <template v-else>
        <span class="mx-auto grid h-12 w-12 place-items-center rounded-xl bg-red-50 text-red-600 ring-1 ring-red-100">
          <svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
        </span>
        <h1 class="mt-4 text-xl font-bold tracking-tight">A belépés nem sikerült</h1>
        <p class="mt-3 text-sm text-red-700">{{ error }}</p>
        <RouterLink class="btn btn-primary mt-6" to="/login">Új link kérése</RouterLink>
      </template>
    </div>
  </section>
</template>
