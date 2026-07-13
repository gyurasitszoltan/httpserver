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
  <section class="mx-auto max-w-md rounded-xl bg-white p-6 text-center shadow-sm ring-1 ring-slate-200">
    <p v-if="!error" class="text-slate-600">Belépés ellenőrzése…</p>
    <template v-else>
      <h1 class="text-xl font-bold">A belépés nem sikerült</h1>
      <p class="mt-3 text-sm text-red-700">{{ error }}</p>
      <RouterLink class="mt-5 inline-block rounded bg-brand-600 px-4 py-2 text-white" to="/login">Új link kérése</RouterLink>
    </template>
  </section>
</template>
