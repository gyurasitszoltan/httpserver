<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { auth } from '../auth'

const user = auth.user
const isAdmin = auth.isAdmin

const initials = computed(() => {
  const src = (user.value?.displayName || user.value?.email || '?').trim()
  return src.split(/[\s@]+/).filter(Boolean).slice(0, 2).map(p => p[0]?.toUpperCase()).join('')
})
</script>

<template>
  <section v-if="user" class="card overflow-hidden">
    <div class="h-1.5 bg-gradient-to-r from-brand-500 via-violet-500 to-fuchsia-400"></div>
    <div class="p-8 sm:p-10">
      <div class="flex flex-col items-start gap-6 sm:flex-row sm:items-center">
        <span class="grid h-16 w-16 shrink-0 place-items-center rounded-2xl bg-gradient-to-br from-brand-500 to-violet-600 text-xl font-bold text-white shadow-lg shadow-brand-500/30">{{ initials }}</span>
        <div>
          <h1 class="text-2xl font-bold tracking-tight sm:text-3xl">Szia{{ user.displayName ? `, ${user.displayName}` : '' }}! 👋</h1>
          <p class="mt-1.5 flex flex-wrap items-center gap-2 text-slate-600">
            Sikeresen be vagy jelentkezve
            <span class="inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-medium text-slate-600 ring-1 ring-slate-200">
              <svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>
              {{ user.email }}
            </span>
          </p>
        </div>
      </div>
      <RouterLink v-if="isAdmin" class="btn btn-primary mt-8" to="/admin/users">
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
        Felhasználók kezelése
      </RouterLink>
    </div>
  </section>
</template>
