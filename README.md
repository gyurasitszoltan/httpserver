# Magic-link user admin

Rust/Axum backend és Vue 3/Vite admin felület. A felhasználók, magic-link tokenek és sessionök SQLite-ban vannak. Az e-mailt a Postmark HTTP API küldi.

## Indítás fejlesztéshez

1. Konfiguráció létrehozása:

   ```sh
   cp .env.example .env
   ```

2. Állítsd be a `.env`-ben a `POSTMARK_SERVER_TOKEN` értékét, valamint egy Postmarkban hitelesített `POSTMARK_FROM` feladót. A `BOOTSTRAP_ADMIN_EMAIL` az első indításkor automatikusan admin felhasználóvá válik.

3. Indítsd külön terminálokban a backendet és frontendet:

   ```sh
   cargo run
   cd frontend && npm install && npm run dev
   ```

   A Vite a `http://localhost:5173` címen érhető el, és az `/api` hívásokat a Rust szerver felé proxyzza.

4. Production buildhez:

   ```sh
   cd frontend && npm run build
   cd .. && cargo run --release
   ```

   A backend a `frontend/dist` könyvtárból szolgálja ki a Vue SPA-t.

## Ellenőrzések

```sh
cargo fmt --check
cargo test
cd frontend && npm run build
```

## API

- `POST /api/auth/magic-link` – magic link igénylése
- `POST /api/auth/verify-magic-link` – token beváltása és session létrehozása
- `POST /api/auth/logout`, `GET /api/auth/me`
- `GET|POST /api/admin/users`
- `GET|PATCH|DELETE /api/admin/users/:id` – admin jogosultsággal
- `GET /healthz`

A `DELETE` soft delete: deaktiválja a felhasználót és a `session_version` módosításával azonnal érvényteleníti a meglévő sessionjeit.
