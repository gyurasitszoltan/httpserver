<script setup lang="ts">
import { ref } from 'vue'
import { api } from '../api'

const email = ref('')
const submitting = ref(false)
const success = ref(false)
const error = ref('')

async function submit() {
  error.value = ''
  submitting.value = true
  try {
    await api.requestMagicLink(email.value)
    success.value = true
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'A kérés sikertelen.'
  } finally { submitting.value = false }
}
</script>

<template>
  <section class="card mx-auto max-w-md overflow-hidden">
    <div class="h-1.5 bg-gradient-to-r from-brand-500 via-violet-500 to-fuchsia-400"></div>
    <div class="p-8">
      <span class="grid h-12 w-12 place-items-center rounded-xl bg-brand-50 text-brand-600 ring-1 ring-brand-100">
        <svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>
      </span>
      <h1 class="mt-5 text-2xl font-bold tracking-tight">Belépés</h1>
      <p class="mt-2 text-sm leading-relaxed text-slate-600">Jelszó helyett küldünk egy egyszer használatos belépési linket.</p>

      <form v-if="!success" class="mt-6 space-y-4" @submit.prevent="submit">
        <label class="block text-sm font-medium text-slate-700">E-mail cím
          <input v-model.trim="email" type="email" required autocomplete="email" placeholder="pelda@ceg.hu" class="input mt-1.5" />
        </label>
        <p v-if="error" class="flex items-center gap-2 rounded-lg bg-red-50 px-3.5 py-2.5 text-sm text-red-700 ring-1 ring-red-100" role="alert">
          <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
          {{ error }}
        </p>
        <button :disabled="submitting" class="btn btn-primary w-full">
          <svg v-if="submitting" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 0 1 8-8v4a4 4 0 0 0-4 4H4z"/></svg>
          {{ submitting ? 'Küldés…' : 'Belépési link küldése' }}
        </button>
      </form>

      <div v-else class="mt-6 flex items-start gap-3 rounded-xl bg-emerald-50 p-4 ring-1 ring-emerald-100">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>
        <p class="text-sm leading-relaxed text-emerald-800">Ha a cím jogosult, elküldtük a belépési linket. Nézd meg a postaládádat!</p>
      </div>
    </div>
  </section>
</template>
