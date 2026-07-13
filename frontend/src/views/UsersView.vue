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
      <div><h1 class="text-2xl font-bold">Felhasználók</h1><p class="text-sm text-slate-600">{{ total }} felhasználó</p></div>
      <button class="rounded bg-brand-600 px-4 py-2 font-medium text-white hover:bg-indigo-700" @click="openCreate">Új felhasználó</button>
    </div>
    <form class="mt-6 flex gap-2" @submit.prevent="load(1)">
      <input v-model="query" class="w-full max-w-md rounded border border-slate-300 px-3 py-2" placeholder="Keresés e-mail vagy név alapján" />
      <button class="rounded border border-slate-300 px-4 py-2 hover:bg-slate-100">Keresés</button>
    </form>
    <p v-if="error" class="mt-4 text-sm text-red-700" role="alert">{{ error }}</p>

    <div class="mt-6 overflow-x-auto rounded-lg bg-white shadow-sm ring-1 ring-slate-200">
      <table class="w-full min-w-[700px] text-left text-sm"><thead class="bg-slate-50 text-slate-600"><tr><th class="p-3">Név / e-mail</th><th class="p-3">Szerepkör</th><th class="p-3">Státusz</th><th class="p-3"></th></tr></thead>
        <tbody><tr v-for="user in users" :key="user.id" class="border-t border-slate-100"><td class="p-3"><div>{{ user.displayName || '—' }}</div><div class="text-slate-500">{{ user.email }}</div></td><td class="p-3">{{ user.role }}</td><td class="p-3"><span :class="user.isActive ? 'bg-emerald-100 text-emerald-800' : 'bg-slate-200 text-slate-700'" class="rounded-full px-2 py-1 text-xs">{{ user.isActive ? 'aktív' : 'inaktív' }}</span></td><td class="p-3 text-right"><button class="mr-3 text-brand-600 hover:underline" @click="openEdit(user)">Szerkesztés</button><button v-if="user.isActive && user.id !== currentUser?.id" class="text-red-700 hover:underline" @click="deactivate(user)">Deaktiválás</button></td></tr>
          <tr v-if="!loading && users.length === 0"><td colspan="4" class="p-8 text-center text-slate-500">Nincs találat.</td></tr>
        </tbody>
      </table>
    </div>
    <div class="mt-4 flex items-center gap-3"><button :disabled="page <= 1" class="rounded border px-3 py-1 disabled:opacity-40" @click="load(page - 1)">Előző</button><span class="text-sm">{{ page }}. oldal</span><button :disabled="page * 25 >= total" class="rounded border px-3 py-1 disabled:opacity-40" @click="load(page + 1)">Következő</button></div>

    <div v-if="showForm" class="fixed inset-0 grid place-items-center bg-slate-900/30 p-4" @click.self="showForm = false">
      <form class="w-full max-w-md space-y-4 rounded-xl bg-white p-6 shadow-xl" @submit.prevent="save">
        <h2 class="text-lg font-bold">{{ editing ? 'Felhasználó szerkesztése' : 'Új felhasználó' }}</h2>
        <label class="block text-sm font-medium">E-mail<input v-model.trim="form.email" :disabled="!!editing" required type="email" class="mt-1 w-full rounded border border-slate-300 px-3 py-2 disabled:bg-slate-100" /></label>
        <label class="block text-sm font-medium">Megjelenített név<input v-model.trim="form.displayName" class="mt-1 w-full rounded border border-slate-300 px-3 py-2" /></label>
        <label class="block text-sm font-medium">Szerepkör<select v-model="form.role" class="mt-1 w-full rounded border border-slate-300 px-3 py-2"><option value="user">user</option><option value="admin">admin</option></select></label>
        <label class="flex items-center gap-2 text-sm"><input v-model="form.isActive" type="checkbox" /> Aktív</label>
        <div class="flex justify-end gap-3"><button type="button" class="rounded px-3 py-2 hover:bg-slate-100" @click="showForm = false">Mégse</button><button class="rounded bg-brand-600 px-4 py-2 text-white">Mentés</button></div>
      </form>
    </div>
  </section>
</template>
