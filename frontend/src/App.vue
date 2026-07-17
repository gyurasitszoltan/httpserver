<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { auth } from './auth'

const router = useRouter()
const user = auth.user
const isAdmin = auth.isAdmin

const initials = computed(() => {
  const src = (user.value?.displayName || user.value?.email || '?').trim()
  return src.split(/[\s@]+/).filter(Boolean).slice(0, 2).map(p => p[0]?.toUpperCase()).join('')
})

async function logout() {
  await auth.logout()
  await router.push('/login')
}
</script>

<template>
  <header class="sticky top-0 z-40 border-b border-slate-200/70 bg-white/70 backdrop-blur-md">
    <nav class="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
      <RouterLink class="group flex items-center gap-2.5" to="/">
        <span class="grid h-8 w-8 place-items-center rounded-lg bg-gradient-to-br from-brand-500 to-violet-600 text-white shadow-sm shadow-brand-500/40 transition-transform duration-200 group-hover:scale-105">
          <svg class="h-4.5 w-4.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2 3 14h7l-1 8 10-12h-7l1-8z"/></svg>
        </span>
        <span class="bg-gradient-to-r from-brand-600 to-violet-600 bg-clip-text text-[15px] font-bold tracking-tight text-transparent">Admin alkalmazás</span>
      </RouterLink>

      <div v-if="user" class="flex items-center gap-2 text-sm sm:gap-4">
        <RouterLink
          v-if="isAdmin"
          to="/admin/users"
          class="rounded-full px-3.5 py-1.5 font-medium text-slate-600 transition-colors hover:bg-slate-100 hover:text-slate-900"
          active-class="bg-brand-50 text-brand-700 hover:bg-brand-50 hover:text-brand-700"
        >Felhasználók</RouterLink>

        <span class="hidden items-center gap-2.5 sm:flex">
          <span class="grid h-8 w-8 place-items-center rounded-full bg-gradient-to-br from-brand-500 to-violet-600 text-xs font-bold text-white shadow-sm">{{ initials }}</span>
          <span class="max-w-40 truncate text-slate-500">{{ user.email }}</span>
        </span>

        <button class="btn btn-ghost !rounded-full !px-3.5 !py-1.5" @click="logout">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="m16 17 5-5-5-5"/><path d="M21 12H9"/></svg>
          Kijelentkezés
        </button>
      </div>
    </nav>
  </header>

  <main class="mx-auto max-w-6xl px-4 py-10 sm:px-6">
    <RouterView v-slot="{ Component }">
      <Transition name="page" mode="out-in">
        <component :is="Component" />
      </Transition>
    </RouterView>
  </main>
</template>
