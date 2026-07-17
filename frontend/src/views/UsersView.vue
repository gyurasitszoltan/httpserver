<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { api, type User } from '../api'
import { auth } from '../auth'

const users = ref<User[]>([])
const currentUser = auth.user
const total = ref(0)
const page = ref(1)
const query = ref('')
const loading = ref(false)
const error = ref('')
const editing = ref<User | null>(null)
const showForm = ref(false)
const form = reactive({ email: '', displayName: '', role: 'user', isActive: true })

const avatarHues = [
  'from-brand-500 to-violet-600',
  'from-sky-500 to-brand-600',
  'from-fuchsia-500 to-purple-600',
  'from-emerald-500 to-teal-600',
  'from-amber-500 to-orange-600',
  'from-rose-500 to-pink-600',
]
function avatarClass(key: string) {
  let h = 0
  for (const c of key) h = (h * 31 + c.charCodeAt(0)) >>> 0
  return avatarHues[h % avatarHues.length]
}
function initials(user: User) {
  const src = (user.displayName || user.email).trim()
  return src.split(/[\s@]+/).filter(Boolean).slice(0, 2).map(p => p[0]?.toUpperCase()).join('')
}

async function load(nextPage = page.value) {
  loading.value = true; error.value = ''
  try {
    const result = await api.users(nextPage, query.value)
    users.value = result.items; total.value = result.total; page.value = result.page
  } catch (e) { error.value = e instanceof Error ? e.message : 'Betöltési hiba.' }
  finally { loading.value = false }
}
function openCreate() {
  editing.value = null
  showForm.value = true
  Object.assign(form, { email: '', displayName: '', role: 'user', isActive: true })
}
function openEdit(user: User) {
  editing.value = user
  showForm.value = true
  Object.assign(form, { email: user.email, displayName: user.displayName ?? '', role: user.role, isActive: user.isActive })
}
async function save() {
  error.value = ''
  try {
    if (editing.value) {
      await api.updateUser(editing.value.id, { displayName: form.displayName || null, role: form.role, isActive: form.isActive })
    } else {
      await api.createUser({ email: form.email, displayName: form.displayName || null, role: form.role, isActive: form.isActive })
    }
    editing.value = null
    showForm.value = false
    await load()
  } catch (e) { error.value = e instanceof Error ? e.message : 'Mentési hiba.' }
}
async function deactivate(user: User) {
  if (!confirm(`${user.email} deaktiválása?`)) return
  try { await api.deactivateUser(user.id); await load() }
  catch (e) { error.value = e instanceof Error ? e.message : 'Műveleti hiba.' }
}
onMounted(load)
</script>

<template>
  <section>
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">Felhasználók</h1>
        <p class="mt-1 text-sm text-slate-500">
          <span class="inline-flex items-center rounded-full bg-brand-50 px-2.5 py-0.5 text-xs font-semibold text-brand-700 ring-1 ring-brand-100">{{ total }}</span>
          felhasználó összesen
        </p>
      </div>
      <button class="btn btn-primary" @click="openCreate">
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
        Új felhasználó
      </button>
    </div>

    <form class="mt-6 flex gap-2" @submit.prevent="load(1)">
      <div class="relative w-full max-w-md">
        <svg class="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
        <input v-model="query" class="input !pl-10" placeholder="Keresés e-mail vagy név alapján" />
      </div>
      <button class="btn btn-ghost">Keresés</button>
    </form>

    <p v-if="error" class="mt-4 flex items-center gap-2 rounded-lg bg-red-50 px-3.5 py-2.5 text-sm text-red-700 ring-1 ring-red-100" role="alert">
      <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
      {{ error }}
    </p>

    <div class="card mt-6 overflow-x-auto">
      <table class="w-full min-w-[700px] text-left text-sm">
        <thead>
          <tr class="border-b border-slate-200 bg-slate-50/80 text-xs font-semibold uppercase tracking-wider text-slate-500">
            <th class="px-5 py-3.5">Név / e-mail</th>
            <th class="px-5 py-3.5">Szerepkör</th>
            <th class="px-5 py-3.5">Státusz</th>
            <th class="px-5 py-3.5"><span class="sr-only">Műveletek</span></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100">
          <tr v-for="user in users" :key="user.id" class="transition-colors hover:bg-slate-50/80">
            <td class="px-5 py-3.5">
              <div class="flex items-center gap-3">
                <span class="grid h-9 w-9 shrink-0 place-items-center rounded-full bg-gradient-to-br text-xs font-bold text-white shadow-sm" :class="avatarClass(user.email)">{{ initials(user) }}</span>
                <div>
                  <div class="font-medium text-slate-900">{{ user.displayName || '—' }}</div>
                  <div class="text-slate-500">{{ user.email }}</div>
                </div>
              </div>
            </td>
            <td class="px-5 py-3.5">
              <span class="inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ring-1" :class="user.role === 'admin' ? 'bg-violet-50 text-violet-700 ring-violet-600/20' : 'bg-slate-100 text-slate-600 ring-slate-500/10'">{{ user.role }}</span>
            </td>
            <td class="px-5 py-3.5">
              <span class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1" :class="user.isActive ? 'bg-emerald-50 text-emerald-700 ring-emerald-600/20' : 'bg-slate-100 text-slate-600 ring-slate-500/20'">
                <span class="h-1.5 w-1.5 rounded-full" :class="user.isActive ? 'bg-emerald-500' : 'bg-slate-400'"></span>
                {{ user.isActive ? 'aktív' : 'inaktív' }}
              </span>
            </td>
            <td class="px-5 py-3.5 text-right">
              <button class="rounded-lg px-2.5 py-1.5 text-sm font-medium text-brand-600 transition-colors hover:bg-brand-50" @click="openEdit(user)">Szerkesztés</button>
              <button v-if="user.isActive && user.id !== currentUser?.id" class="ml-1 rounded-lg px-2.5 py-1.5 text-sm font-medium text-red-600 transition-colors hover:bg-red-50" @click="deactivate(user)">Deaktiválás</button>
            </td>
          </tr>
          <tr v-if="!loading && users.length === 0">
            <td colspan="4" class="px-5 py-14 text-center">
              <svg class="mx-auto h-8 w-8 text-slate-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
              <p class="mt-3 text-slate-500">Nincs találat.</p>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="loading" class="flex items-center justify-center gap-2 border-t border-slate-100 py-4 text-sm text-slate-500">
        <svg class="h-4 w-4 animate-spin text-brand-500" viewBox="0 0 24 24" fill="none"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 0 1 8-8v4a4 4 0 0 0-4 4H4z"/></svg>
        Betöltés…
      </div>
    </div>

    <div class="mt-4 flex items-center gap-3">
      <button :disabled="page <= 1" class="btn btn-ghost !px-3 !py-1.5" @click="load(page - 1)">
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
        Előző
      </button>
      <span class="text-sm text-slate-500">{{ page }}. oldal</span>
      <button :disabled="page * 25 >= total" class="btn btn-ghost !px-3 !py-1.5" @click="load(page + 1)">
        Következő
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
      </button>
    </div>

    <Transition name="modal">
      <div v-if="showForm" class="fixed inset-0 z-50 grid place-items-center bg-slate-900/40 p-4 backdrop-blur-sm" @click.self="showForm = false">
        <form class="modal-panel w-full max-w-md space-y-4 rounded-2xl bg-white p-6 shadow-2xl ring-1 ring-slate-900/5" @submit.prevent="save">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-bold tracking-tight">{{ editing ? 'Felhasználó szerkesztése' : 'Új felhasználó' }}</h2>
            <button type="button" class="grid h-8 w-8 place-items-center rounded-lg text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600" @click="showForm = false">
              <svg class="h-4.5 w-4.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
          </div>
          <label class="block text-sm font-medium text-slate-700">E-mail
            <input v-model.trim="form.email" :disabled="!!editing" required type="email" class="input mt-1.5" />
          </label>
          <label class="block text-sm font-medium text-slate-700">Megjelenített név
            <input v-model.trim="form.displayName" class="input mt-1.5" />
          </label>
          <label class="block text-sm font-medium text-slate-700">Szerepkör
            <select v-model="form.role" class="input mt-1.5">
              <option value="user">user</option>
              <option value="admin">admin</option>
            </select>
          </label>
          <label class="flex cursor-pointer items-center gap-2.5 text-sm text-slate-700">
            <input v-model="form.isActive" type="checkbox" class="h-4 w-4 rounded border-slate-300 text-brand-600 accent-brand-600" />
            Aktív
          </label>
          <div class="flex justify-end gap-3 pt-2">
            <button type="button" class="btn btn-ghost" @click="showForm = false">Mégse</button>
            <button class="btn btn-primary">Mentés</button>
          </div>
        </form>
      </div>
    </Transition>
  </section>
</template>
