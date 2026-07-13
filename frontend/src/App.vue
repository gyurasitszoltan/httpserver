<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { auth } from './auth'

const router = useRouter()
const user = auth.user
const isAdmin = auth.isAdmin
async function logout() {
  await auth.logout()
  await router.push('/login')
}
</script>

<template>
  <header class="border-b border-slate-200 bg-white">
    <nav class="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
      <RouterLink class="font-semibold text-brand-600" to="/">Admin alkalmazás</RouterLink>
      <div v-if="user" class="flex items-center gap-3 text-sm">
        <RouterLink v-if="isAdmin" class="text-slate-600 hover:text-brand-600" to="/admin/users">Felhasználók</RouterLink>
        <span class="hidden text-slate-500 sm:inline">{{ user.email }}</span>
        <button class="rounded bg-slate-100 px-3 py-1.5 hover:bg-slate-200" @click="logout">Kijelentkezés</button>
      </div>
    </nav>
  </header>
  <main class="mx-auto max-w-5xl px-4 py-10"><RouterView /></main>
</template>
