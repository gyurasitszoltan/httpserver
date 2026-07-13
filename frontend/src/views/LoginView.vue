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
  <section class="mx-auto max-w-md rounded-xl bg-white p-6 shadow-sm ring-1 ring-slate-200">
    <h1 class="text-2xl font-bold">Belépés</h1>
    <p class="mt-2 text-sm text-slate-600">Jelszó helyett küldünk egy egyszer használatos belépési linket.</p>
    <form v-if="!success" class="mt-6 space-y-4" @submit.prevent="submit">
      <label class="block text-sm font-medium">E-mail cím
        <input v-model.trim="email" type="email" required autocomplete="email" class="mt-1 block w-full rounded border border-slate-300 px-3 py-2 outline-none focus:border-brand-600 focus:ring-2 focus:ring-brand-600/20" />
      </label>
      <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
      <button :disabled="submitting" class="w-full rounded bg-brand-600 px-4 py-2 font-medium text-white hover:bg-indigo-700 disabled:opacity-60">
        {{ submitting ? 'Küldés…' : 'Belépési link küldése' }}
      </button>
    </form>
    <p v-else class="mt-6 rounded bg-emerald-50 p-4 text-sm text-emerald-800">Ha a cím jogosult, elküldtük a belépési linket.</p>
  </section>
</template>
