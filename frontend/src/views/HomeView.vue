<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { auth } from '../auth'
const user = auth.user
const isAdmin = auth.isAdmin
</script>

<template>
  <section v-if="user" class="rounded-xl bg-white p-8 shadow-sm ring-1 ring-slate-200">
    <h1 class="text-2xl font-bold">Szia{{ user.displayName ? `, ${user.displayName}` : '' }}!</h1>
    <p class="mt-2 text-slate-600">Sikeresen be vagy jelentkezve: {{ user.email }}</p>
    <RouterLink v-if="isAdmin" class="mt-6 inline-block rounded bg-brand-600 px-4 py-2 font-medium text-white hover:bg-indigo-700" to="/admin/users">Felhasználók kezelése</RouterLink>
  </section>
</template>
