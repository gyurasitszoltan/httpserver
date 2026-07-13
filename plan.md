# Általános alkalmazás – implementációs terv

## 1. Cél és hatókör

Egy egylapos Vue alkalmazás Rust REST backenddel, amelyben:

- a felhasználók jelszó helyett e-mailben kapott, egyszer használatos **magic linkkel** jelentkeznek be;
- a bejelentkezett felhasználók sessionje SQLite-ban tárolódik;
- csak admin szerepkörű felhasználó érheti el a user admin panelt;
- az admin panel REST API-n keresztül kezeli a felhasználókat (lista, létrehozás, módosítás, törlés);
- minden futási konfiguráció `.env` fájlból származik.

A rendszer kezdetben egyetlen alkalmazásként fut: az Axum szolgálja ki az API-t, production buildben pedig a Vite által elkészített statikus frontendet is.

## 2. Technológiai döntések

### Backend

| Terület | Eszköz | Szerep |
|---|---|---|
| Async runtime | `tokio` | HTTP szerver, háttérfeladatok |
| HTTP / routing | `axum` | route-ok, extractors, response-ok |
| Bejelentkezés és authorization | `axum-login` | `AuthUser`, `AuthnBackend`, `AuthSession`, role-ellenőrzés |
| Middleware | `tower-http` | trace, CORS, request ID, security header, statikus fájlok |
| Adatbázis | SQLite + `sqlx` | pool, migrációk, típusbiztos lekérdezések |
| Szerializáció | `serde`, `serde_json` | API DTO-k |
| E-mail | `postmark` (`reqwest` feature) | magic link kiküldése a Postmark HTTP API-n |
| Token | `rand` + `sha2` | kriptográfiailag véletlen token, csak hash kerül adatbázisba |
| Konfiguráció | `dotenvy` | `.env` betöltése fejlesztéskor |
| Hibakezelés / naplózás | `thiserror`, `tracing`, `tracing-subscriber` | egységes API-hibák és auditálható logok |

A `sqlx` SQLite feature-je legyen bekapcsolva. A kapcsolati string `sqlite://...` formátumú, a pool indításkor migráció után jön létre. SQLite esetén engedélyezni kell a foreign key constraintet és érdemes WAL journal módot beállítani.

### Frontend

| Terület | Eszköz | Szerep |
|---|---|---|
| Build / dev server | Vite | Vue fejlesztés és production build |
| UI | Vue 3 + TypeScript + Composition API | oldalak, komponensek, állapot |
| Navigáció | Vue Router | publikus, authentikált és admin route-ok |
| Stílus | Tailwind CSS | reszponzív UI |
| HTTP kliens | natív `fetch` (vagy kicsi wrapper) | REST hívások, egységes hibakezelés |

A frontend fejlesztői módban a Vite proxyval az `/api` kéréseket a Rust szerverre továbbítja. Productionben relatív `/api/...` URL-ek maradnak, így nincs külön API originből adódó CORS-probléma.

## 3. Javasolt könyvtárstruktúra

```text
.
├── .env.example
├── Cargo.toml
├── migrations/
│   ├── 0001_initial.sql
│   └── 0002_seed_first_admin.sql       # opcionális, csak fejlesztéshez
├── src/
│   ├── main.rs                          # konfiguráció, pool, router, szerver
│   ├── config.rs
│   ├── error.rs
│   ├── state.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   ├── magic_links.rs
│   │   └── sessions.rs
│   ├── auth/
│   │   ├── mod.rs                       # axum-login backend implementáció
│   │   ├── user.rs                      # AuthUser implementáció
│   │   ├── magic_link.rs
│   │   └── authorization.rs             # require_auth / require_admin
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── me.rs
│   │   └── admin_users.rs
│   └── email.rs
├── frontend/
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts
│       ├── router.ts
│       ├── api/client.ts
│       ├── stores/auth.ts
│       ├── views/LoginView.vue
│       ├── views/MagicLinkCallbackView.vue
│       ├── views/HomeView.vue
│       ├── views/admin/UsersView.vue
│       └── components/admin/UserForm.vue
└── dist/                                # Vite build kimenet, nem verziózott
```

## 4. Adatmodell és migrációk

Minden időbélyeg UTC ISO-8601/RFC3339 szöveg vagy SQLite `DATETIME`; az egész projekten belül egységesen ugyanazt a formátumot kell használni.

### `users`

| Mező | Típus / szabály | Leírás |
|---|---|---|
| `id` | INTEGER PK | belső azonosító |
| `email` | TEXT UNIQUE NOT NULL, normalizált | belépési azonosító |
| `display_name` | TEXT NULL | admin által szerkeszthető név |
| `role` | TEXT NOT NULL | `admin` vagy `user`, CHECK constrainttel |
| `is_active` | INTEGER NOT NULL DEFAULT 1 | tiltott felhasználó nem jelentkezhet be |
| `created_at` | DATETIME NOT NULL | létrehozás ideje |
| `updated_at` | DATETIME NOT NULL | módosítás ideje |
| `last_login_at` | DATETIME NULL | sikeres magic-link belépés |

Az e-mail trimelt és kisbetűs formában kerül tárolásra. A magic link kérése csak előre felvett, aktív usernek küld linket; ezzel az admin kontrollálja a hozzáférést.

### `magic_link_tokens`

| Mező | Típus / szabály | Leírás |
|---|---|---|
| `id` | INTEGER PK | azonosító |
| `user_id` | INTEGER NOT NULL FK `users(id)` | igénylő felhasználó |
| `token_hash` | TEXT UNIQUE NOT NULL | a nyers token SHA-256 hash-e |
| `expires_at` | DATETIME NOT NULL | rövid, pl. 15 perces lejárat |
| `used_at` | DATETIME NULL | egyszer használhatóság |
| `created_at` | DATETIME NOT NULL | igénylés ideje |
| `request_ip` | TEXT NULL | opcionális audit / rate limit adat |

Index kell `token_hash`-re, `user_id`-ra és lejárt tokenek takarításához `expires_at`-ra. Új link igénylésekor a user korábbi felhasználatlan tokenjeit érvényteleníteni kell.

### `user_sessions`

Az `axum-login` session store-ja SQLite alapú legyen. A session rekord legalább a session azonosítót, a szerializált session adatot és a lejáratot tartalmazza. A session cookie-ban csak egy véletlen session ID szerepel; a felhasználói adat és az authorization állapot szerveroldali SQLite-ban marad.

Ha az alkalmazott `axum-login` / `tower-sessions` verzió saját SQLite store-ot ad, annak hivatalos sémáját kell migrációban használni; ellenkező esetben egy kompatibilis `tower-sessions-sqlx-store` SQLite store-t kell használni. A store élettartamát és cookie nevét explicit konfigurálni kell, nem in-memory defaulttal futtatni.

### Opcionális `audit_log`

Ajánlott táblázat admin műveletekhez: `actor_user_id`, `action`, `target_user_id`, `metadata_json`, `created_at`. Legalább user létrehozás, módosítás, deaktiválás/törlés és szerepváltoztatás kerüljön bele.

## 5. Konfiguráció (`.env`)

A konfigurációt egy `Config` struktúra olvassa be induláskor, validálja, majd immutable `AppState` része lesz. Titkok nem kerülhetnek forráskódba vagy kliens bundle-be.

```dotenv
# Szerver és adatbázis
APP_ENV=development
BIND_ADDR=127.0.0.1:3000
DATABASE_URL=sqlite://./data/app.db?mode=rwc
APP_BASE_URL=http://localhost:3000
FRONTEND_DIST_DIR=./frontend/dist

# Session cookie
SESSION_COOKIE_NAME=app_session
SESSION_SECRET=legalabb_64_karakteres_veletlen_titok
SESSION_SECURE=false
SESSION_SAME_SITE=lax
SESSION_MAX_AGE_HOURS=168

# Magic link
MAGIC_LINK_TTL_MINUTES=15
MAGIC_LINK_REQUESTS_PER_HOUR=5

# Postmark e-mail API
POSTMARK_SERVER_TOKEN=postmark_server_token
POSTMARK_FROM=No Reply <no-reply@example.com>
POSTMARK_MESSAGE_STREAM=outbound

# Első admin bootstrap (csak induláskor, ha még nincs admin)
BOOTSTRAP_ADMIN_EMAIL=admin@example.test
```

A `.env` és az SQLite adatfájl legyen `.gitignore`-ban. A verziózott `.env.example` csak dokumentált, biztonságos placeholder értékeket tartalmazzon. Productionben környezeti változók vagy titokkezelő felülírhatják a `.env`-et.

## 6. Hitelesítési és authorization folyamat

### Magic link igénylése

1. A kliens elküldi az e-mail címet a `POST /api/auth/magic-link` végpontra.
2. A backend normalizálja a címet, IP- és e-mail-alapú rate limitet ellenőriz.
3. A válasz minden esetben azonos, pl. `202 Accepted` és „Ha a cím jogosult, elküldtük a linket”, így nem szivárog ki, létezik-e user.
4. Aktív user esetén a backend `rand` segítségével elég hosszú véletlen tokent generál, adatbázisba csak a hash kerül, majd a nyers tokenből készít URL-t: `APP_BASE_URL/auth/callback?token=...`.
5. Az e-mail küldést a Postmark Server API tokennel kell indítani; a küldési hiba és a Postmark válasz Message ID-ja naplózandó, de a kliens nem kap szolgáltató-specifikus részleteket.

### Magic link beváltása

1. A Vue callback oldal kiolvassa a tokent és `POST /api/auth/verify-magic-link` hívással elküldi.
2. Egy adatbázis tranzakcióban a backend megkeresi a hash-t, ellenőrzi a lejáratot, a `used_at IS NULL` állapotot és az aktív usert, majd a tokent felhasználtnak jelöli.
3. A backend `axum-login` `AuthSession` segítségével belépteti a usert. A session rekord SQLite-ba kerül, a kliens pedig csak `HttpOnly` cookie-t kap.
4. A frontend `GET /api/auth/me` hívással frissíti a kliensoldali auth állapotot, majd a főoldalra irányít.
5. Érvénytelen vagy lejárt token esetén a kliens általános hibaüzenetet kap, új linket kérhet.

### Session és szerepkör ellenőrzése

- `AuthUser` a `users` rekordot reprezentálja, egyedi ID-val és hitelesített sessionből visszaolvasható azonosítóval.
- Az autentikációt igénylő handler a sessionből betöltött usert várja; hiány esetén `401` JSON hiba.
- Az admin route-ok közös `require_admin` extractorral/middleware-rel ellenőrzik, hogy `role == "admin"` és `is_active == true`; hiba esetén `403`.
- Kijelentkezéskor a `POST /api/auth/logout` kiüríti és törli a szerveroldali sessiont, majd lejáratja a cookie-t.
- Szerep módosításakor vagy user tiltásakor az adott user összes meglévő sessionjét törölni kell. Így a jogosultság azonnal érvénybe lép.

## 7. REST API szerződés

Minden API válasz JSON. Hibaforma:

```json
{ "error": { "code": "forbidden", "message": "Nincs jogosultságod ehhez a művelethez." } }
```

### Auth és saját profil

| Metódus | Útvonal | Jogosultság | Kérés / válasz |
|---|---|---|---|
| `POST` | `/api/auth/magic-link` | publikus | `{ "email": "..." }` → `202` |
| `POST` | `/api/auth/verify-magic-link` | publikus | `{ "token": "..." }` → aktuális user + session cookie |
| `POST` | `/api/auth/logout` | bejelentkezett | `204` |
| `GET` | `/api/auth/me` | bejelentkezett | `{ "id", "email", "displayName", "role" }` |

### Admin user CRUD

Minden `/api/admin/*` végpont admin jogosultságot követel.

| Metódus | Útvonal | Funkció |
|---|---|---|
| `GET` | `/api/admin/users?page=1&pageSize=25&query=` | lapozott, e-mail/név szerinti kereshető lista |
| `POST` | `/api/admin/users` | user létrehozása (`email`, `displayName`, `role`, `isActive`) |
| `GET` | `/api/admin/users/:id` | egy user lekérése |
| `PATCH` | `/api/admin/users/:id` | név, szerepkör, aktív állapot módosítása |
| `DELETE` | `/api/admin/users/:id` | user törlése vagy ajánlottan soft-delete/deaktiválás |

A listaválasz példája:

```json
{
  "items": [{ "id": 1, "email": "admin@example.test", "displayName": "Admin", "role": "admin", "isActive": true }],
  "page": 1,
  "pageSize": 25,
  "total": 1
}
```

Validáció: e-mail formátum és maximum hossz, megengedett szerepkörök, biztonságos `pageSize` felső határ. Duplikált e-mail `409 Conflict`, nem létező rekord `404`.

Üzleti védelmek:

- az admin nem deaktiválhatja vagy törölheti saját magát;
- nem törölhető/deaktiválható az utolsó aktív admin;
- role adminról userre váltásnál ugyanez az „utolsó admin” szabály él;
- user törlés/tiltás előtt a hozzá tartozó sessionök és aktív magic linkek érvénytelenednek.

## 8. Frontend képernyők és viselkedés

1. **Belépés (`/login`)**: e-mail mező, linkküldés gomb, semleges sikerüzenet, kliensoldali formátumvalidáció.
2. **Magic link callback (`/auth/callback`)**: betöltés közben beváltja a tokent; siker után `/`, hiba esetén `/login` és új link kérési lehetőség. A token az URL-ből azonnal eltávolítandó (`replace`) a sikeres hívás után.
3. **Főoldal (`/`)**: aktuális user és kijelentkezés. Az adminnak admin panel link.
4. **Admin felhasználók (`/admin/users`)**: kereső, lapozott táblázat, létrehozó/szerkesztő modal vagy külön oldal, státusz és szerepkör jelölők, törlés/deaktiválás megerősítő dialógus.
5. **404 / jogosultsághiány**: értelmes hibaoldalak.

A Vue Router route guardja a `GET /api/auth/me` által betöltött auth store alapján terel `/login` felé, illetve admin route-ról főoldalra vagy 403 oldalra. A guard csak UX-réteg: a valódi jogosultságellenőrzés kizárólag a backendben történik.

Az API kliens minden hívásnál `credentials: "include"` opciót használ. `401` esetén törli a kliensoldali auth state-et és belépésre irányít; strukturált API hibát jelenít meg, nem nyers szerverüzenetet.

## 9. Router, middleware és biztonság

Javasolt route-sorrend:

1. publikus `/api/auth/magic-link` és `/api/auth/verify-magic-link`;
2. session middleware-rel védett `/api/auth/logout` és `/api/auth/me`;
3. session + `require_admin` alatt `/api/admin/users` CRUD;
4. productionben statikus frontend és SPA fallback, amely nem nyel el `/api/*` 404-eket.

`tower-http` middleware-ek:

- `TraceLayer`: kérés ID-val, metódus/útvonal/státusz/időtartam logolása; token, cookie és e-mail nem naplózható;
- `RequestIdLayer` és válasz request ID;
- `CorsLayer`: fejlesztéskor kizárólag a Vite originje, cookie miatt `allow_credentials(true)`; productionben lehetőleg azonos origin;
- `SetResponseHeaderLayer`: `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, clickjacking elleni `X-Frame-Options: DENY` és CSP;
- request body méretlimit és timeout;
- rate limit a magic-link kérésekre, lehetőleg tartósan SQLite-ban vagy proxy/API gateway szinten. Egyetlen processzes fejlesztői limit lehet memóriás, de production több példány esetén ne az legyen az egyetlen védelem.

Cookie szabályok: `HttpOnly`, `Secure` productionben, explicit `SameSite=Lax` (vagy az integráció igénye szerint `Strict`), szűk `Path=/`, rövid és konfigurált session lejárat. A frontend soha nem olvas session tokent JavaScriptből.

## 10. Implementációs lépések

1. **Projekt alapozás**
   - Rust függőségek felvétele a szükséges feature-ökkel; `frontend/` Vite Vue TypeScript projekttel és Tailwinddel.
   - `.env.example`, `.gitignore` (`.env`, `data/*.db`, `frontend/dist`) és a könyvtárstruktúra létrehozása.
   - `Config` betöltése és fail-fast validációja (kötelező titkok, URL-ek, TTL-ek).

2. **SQLite réteg**
   - `sqlx` pool és migrációfuttatás startupkor.
   - `users`, `magic_link_tokens`, session store és opcionális audit migrációk.
   - repository függvények: user keresés/listázás, tranzakciós token-beváltás, session-érvénytelenítés, admin invariánsok.

3. **Auth infrastruktúra**
   - `AuthUser` és `AuthnBackend` implementálása `axum-login`-hoz.
   - SQLite-alapú session store és cookie konfiguráció bekötése.
   - `require_auth` / `require_admin` és egységes `AppError` → JSON hiba mapping.

4. **Magic link és e-mail**
   - Biztonságos token generálás/hash-elés, TTL, egyszer használhatóság és takarító feladat.
   - `postmark` kliens (`reqwest` feature), szöveges és HTML e-mail sablon, linkkódolás. A `PostmarkClient` a `POSTMARK_SERVER_TOKEN` értékével épül; egy `SendEmailRequest` a `POSTMARK_FROM`, címzett, tárgy, text/HTML törzs és `POSTMARK_MESSAGE_STREAM` értékét használja.
   - Auth API endpointok, rate limiting, általános válaszok.

5. **Admin API**
   - DTO-k és inputvalidáció.
   - User lista, létrehozás, részlet, módosítás, deaktiválás/törlés.
   - Utolsó admin és saját fiók védelme, session revoke, audit log.

6. **Frontend**
   - API wrapper, auth store és router guardok.
   - Login/callback/főoldal, majd Tailwindes admin lista és CRUD űrlapok.
   - Loading, üres állapot, hibák, mobilnézet és akadálymentes form label/focus kezelés.

7. **Production integráció és megfigyelhetőség**
   - Vite build kiszolgálása Axumból, SPA fallback és fejlesztői proxy.
   - strukturált `tracing` log, health endpoint (`GET /healthz`) adatbázis-ellenőrzéssel.
   - dokumentált indítás, migráció, backup és Postmark beállítás.

## 11. Tesztelési terv és elfogadási kritériumok

### Backend tesztek

- unit: e-mail normalizálás, token hash/lejárat, inputvalidáció, role szabályok;
- repository integration teszt ideiglenes SQLite DB-vel és migrációkkal;
- API integration teszt: magic link kérés nem fed fel userlétezést, lejárt/felhasznált token elutasított, sikeres beváltás sessiont ad;
- authorization teszt: anonim `401`, normál user admin route-on `403`, admin CRUD-ot használhat;
- admin invariánsok: utolsó aktív admin és saját fiók nem vonható vissza, tiltás után régi session nem használható;
- az e-mail küldő adapter mockjával ellenőrizhető, hogy a helyes magic-link URL, feladó, címzett és Message Stream kerül a Postmark-kérésbe; a Server Token nem kerülhet logba vagy tesztkimenetbe.

### Frontend tesztek

- login űrlap állapotai és callback siker/hiba;
- API mockkal auth store és route guard;
- admin lista, keresés, lapozás, form validáció és hibamegjelenítés;
- manuális ellenőrzés: mobil layout, billentyűzetes használat és képernyőolvasó címkék.

### Késznek tekinthető, ha

- a bootstrap admin magic linkkel be tud lépni;
- minden session és auth adat SQLite-ban marad szerver újraindítása után;
- a normál user nem érhet el admin API-t vagy admin UI funkciót;
- az admin felhasználót tud létrehozni, módosítani, listázni és deaktiválni/törölni;
- a magic link rövid életű, egyszer használható, a nyers token nem kerül adatbázisba vagy logba;
- minden konfigurálható érték `.env`/környezeti változóból érkezik, és titok nem szerepel a repositoryban.

## 12. Üzemeltetési megjegyzések

- SQLite fájlról rendszeres, konzisztens backup szükséges; backup előtt használható SQLite online backup vagy rövid karbantartási eljárás.
- Productionben HTTPS kötelező, ezért `SESSION_SECURE=true` és valódi `APP_BASE_URL` kell.
- A régi magic link tokeneket és lejárt sessionöket ütemezetten takarítani kell.
- Az első admin bootstrappelése csak akkor fusson, ha nincs aktív admin; a konfigurációs e-mailt nem szabad minden indulásnál felülírni.
- A Postmarkban a `POSTMARK_FROM` feladónak regisztrált és megerősített Sender Signature-nek vagy hitelesített domainhez tartozó címnek kell lennie. Productionben a megfelelő Postmark Message Streamet, valamint SPF/DKIM/DMARC beállítást kell használni.
