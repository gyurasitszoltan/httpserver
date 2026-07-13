# `main.rs` modularizálási terv

## Cél

A jelenlegi egyfájlos Axum alkalmazás felbontása funkcionális határok mentén úgy, hogy:

- `main.rs` csak az alkalmazás indítását végezze;
- a route-ok, auth, adatbázis, e-mail és validáció külön felelősségi egység legyen;
- **ne legyen `mod.rs`**;
- a mappás modulokhoz felső szintű fájl tartozzon, például `routes.rs` és `routes/`;
- az API viselkedése, endpointjai és adatbázis-sémája változatlan maradjon.

## Célkönyvtár-struktúra

```text
src/
├── main.rs
├── app.rs
├── config.rs
├── error.rs
├── state.rs
├── auth.rs
├── mail.rs
├── validation.rs
├── db.rs
├── db/
│   ├── users.rs
│   └── magic_links.rs
├── dto.rs
├── dto/
│   ├── auth.rs
│   └── users.rs
├── routes.rs
└── routes/
    ├── health.rs
    ├── auth.rs
    └── users.rs
```

A Rustban a `routes.rs` automatikusan a `routes/` mappa gyökérmodulja. Ezért ebben a fájlban szerepel majd például:

```rust
pub mod auth;
pub mod health;
pub mod users;
```

Ugyanez vonatkozik a `db.rs` → `db/` és `dto.rs` → `dto/` párokra is. `mod.rs` nem készül.

## Modulok felelőssége

| Fájl/modul | Tartalom | Nyilvános API |
|---|---|---|
| `main.rs` | tracing inicializálás, konfiguráció betöltés, listener indítás, graceful shutdown | nincs üzleti logika |
| `app.rs` | DB/sessions/state felépítése, router és middleware-ek összeállítása | `build_state`, `router` |
| `config.rs` | `Config`, környezeti változók olvasása és validálása | `Config::load` |
| `error.rs` | `E`, `R<T>`, HTTP hibaválaszok | `E`, `R` |
| `state.rs` | `StateData` | `StateData` |
| `auth.rs` | `User`, `Backend`, `Cred`, `AuthUser`, `AuthnBackend`, `Session` alias | `User`, `Backend`, `Session` |
| `mail.rs` | Postmark kliens és e-mail küldés | `Mail` |
| `validation.rs` | e-mail, szerepkör és név ellenőrzése | `email_of`, `role`, `name` |
| `db/users.rs` | felhasználó lekérdezések és admin-szabályok | `active`, `find`, `bootstrap`, `ensure_not_last_admin` |
| `db/magic_links.rs` | magic-link tokenek tárolása, felhasználása, óránkénti limit | célzott DB-függvények |
| `dto/auth.rs` | auth request/response DTO-k | `LinkReq`, `Verify`, `Me` |
| `dto/users.rs` | user admin request/response/listázási DTO-k | `Create`, `Update`, `ListQ`, `UserOut`, `UserList` |
| `routes/health.rs` | `GET /healthz` | `health` |
| `routes/auth.rs` | magic-link, belépés, kilépés, aktuális user | `link`, `verify`, `logout`, `me` |
| `routes/users.rs` | admin felhasználó CRUD | `list`, `show`, `create`, `update`, `remove` |
| `routes.rs` | route almodulok deklarálása és opcionálisan API-router építése | `api_router` |

## Függőségi irány

A felsőbb rétegek használhatják az alattuk lévőket, fordítva ne legyen függés:

```text
main → app → routes → {state, auth, dto, db, mail, validation, error}
                  ↓
               config
```

Különösen:

- `db/*` ne importáljon `routes/*` vagy HTTP/Axum típusokat;
- `dto/*` csak szerializációs típusokat tartalmazzon, üzleti logika nélkül;
- `validation.rs` ne ismerjen adatbázist vagy HTTP-t;
- a route handler alakítja a `db`- és `mail`-hibákat a közös `E` hibává;
- `app.rs` legyen az egyetlen hely, ahol middleware, CORS, session és static-file kiszolgálás össze van kötve.

## Konkrét áthelyezési terv

### 1. Alap, függőségmentes modulok

Elsőként hozzuk létre az alábbiakat változatlan logikával:

- `config.rs`: a jelenlegi `Config` és `Config::load`;
- `error.rs`: `E`, `R<T>`, `From<sqlx::Error>`, `IntoResponse`;
- `validation.rs`: `email_of`, `role`, `name`;
- `dto.rs` és `dto/auth.rs`, `dto/users.rs`: a request/response struktúrák;
- `state.rs`: `StateData`.

Ezzel a fordítás közben gyorsan felismerhetők az import- és láthatósági hibák, még a route-ok mozgatása előtt.

### 2. Auth és e-mail infrastruktúra

- `auth.rs`-ba kerül: `COLUMNS`, `User`, `Backend`, `Cred`, a két axum-login trait implementáció, `Session` és a sessionből felhasználót/adminisztrátort kinyerő segédfüggvények.
- `mail.rs`-ba kerül: `Mail`, `Mail::new`, `Mail::send`.
- A token-kezelés (`token`, `hash`) maradjon kezdetben `db/magic_links.rs` privát segédfüggvénye vagy kerüljön külön `crypto.rs`-ba, ha más funkció is használni fogja.
- `now()` legyen egyetlen közös segédfüggvény, például `time.rs`-ban. Ha nem indokolt új fájl, ideiglenesen maradhat a két DB modulhoz közel, de ne legyen duplikálva.

### 3. Adatbázis-hozzáférés kiemelése

Hozzuk létre a következő felső modul-fájlokat:

```rust
// src/db.rs
pub mod magic_links;
pub mod users;
```

`db/users.rs` tartalma:

- `active`;
- `find`;
- `bootstrap`;
- a jelenlegi `last` átnevezve: `ensure_not_last_admin`;
- opcionálisan a user-listázás, user-létrehozás és user-frissítés lekérdezései.

`db/magic_links.rs` tartalma:

- óránkénti kérésdarab lekérdezése;
- régi, fel nem használt tokenek érvénytelenítése;
- új token beszúrása;
- token + felhasználó atomi megkeresése és felhasználása tranzakcióban.

A cél, hogy a `routes/auth.rs` leírja a belépési folyamatot, de ne tartalmazzon hosszú nyers SQL blokkot. A tranzakció maradjon a `db/magic_links.rs`-ban, hogy a token egyszer használhatósága egy helyen garantált legyen.

### 4. Route-ok szétválasztása

`routes.rs`:

```rust
pub mod auth;
pub mod health;
pub mod users;

use axum::{routing::{get, post}, Router};
use crate::state::StateData;

pub fn api_router(state: StateData) -> Router {
    Router::new()
        .route("/healthz", get(health::health))
        .route("/api/auth/magic-link", post(auth::link))
        .route("/api/auth/verify-magic-link", post(auth::verify))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/admin/users", get(users::list).post(users::create))
        .route(
            "/api/admin/users/{id}",
            get(users::show).patch(users::update).delete(users::remove),
        )
        .with_state(state)
}
```

A handler-ek áthelyezése:

- `routes/health.rs`: `health`;
- `routes/auth.rs`: `link`, `verify`, `logout`, `me`, illetve a csak itt használt `me_of`;
- `routes/users.rs`: `list`, `show`, `create`, `update`, `remove`.

A handler paraméterei (`State`, `Path`, `Query`, `Json`, `Session`) és a HTTP válaszformátumok ne változzanak.

### 5. App-összeállítás és vékony `main.rs`

`app.rs` feladatai:

1. SQLite könyvtár létrehozása szükség esetén;
2. SQLite pool létrehozása, WAL és foreign key beállításokkal;
3. SQLx migrációk futtatása;
4. bootstrap admin létrehozása;
5. session store és auth layer felépítése;
6. `StateData` létrehozása;
7. `routes::api_router(state)` kiegészítése static file fallbackkel és middleware-ekkel.

A `main.rs` végül csak:

1. tracing inicializálás;
2. `Config::load()`;
3. `app::build(...)`;
4. TCP bind;
5. `axum::serve(...).with_graceful_shutdown(...)`.

## Láthatósági szabályok

- Alapértelmezésben minden elem privát.
- Csak az legyen `pub`, amit egy másik felső szintű modul tényleg használ.
- Az almodulok közötti belső API-nál, ha elegendő, használjunk `pub(crate)`-ot `pub` helyett.
- Ne legyen globális `use crate::*`; minden modul explicit importálja a szükséges típusokat.

## Megvalósítási sorrend és ellenőrzés

1. Kiinduló állapot ellenőrzése: `cargo fmt --check && cargo test`.
2. DTO, config, error, validation és state kiemelése; `cargo check`.
3. Auth és Mail kiemelése; `cargo check`.
4. `db.rs` + `db/` bevezetése, SQL logika áthelyezése; `cargo test`.
5. `routes.rs` + `routes/` bevezetése, endpointok átkötése; `cargo test`.
6. `app.rs` létrehozása és `main.rs` vékonyítása; `cargo fmt && cargo test`.
7. Manuális smoke test:
   - `GET /healthz` → `204`;
   - magic-link kérés ismeretlen címre → `202`;
   - auth nélküli `/api/admin/users` → `401`;
   - nem admin sessionnel admin endpoint → `403`;
   - frontend SPA fallback továbbra is az `index.html`-t adja.

## Nem része ennek a változtatásnak

- endpointok vagy JSON szerződések módosítása;
- adatbázis-migráció vagy séma átalakítása;
- új dependency felvétele;
- repository/service abstraction erőltetése minden egyes SQL lekérdezéshez;
- Docker vagy systemd konfiguráció módosítása.

A cél először a viselkedésváltozás nélküli, könnyebben karbantartható szerkezet. A további domain/service rétegezést csak akkor érdemes bevezetni, ha új funkciók miatt tényleg ismétlődik az üzleti logika.
