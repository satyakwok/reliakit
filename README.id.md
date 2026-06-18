<p align="center">
  <img src="./assets/reliakit-logo.png" alt="Reliakit" width="520">
</p>

[Bahasa Indonesia](./README.id.md) | [English](./README.md)

# Reliakit

Crate Rust untuk reliability yang kecil dan tanpa dependency: invariant yang
eksplisit, secret yang diredaksi, input yang dibatasi, data yang deterministik,
dan resilience yang runtime-agnostic. Ramah `no_std`, tanpa `unsafe`, dipakai
satu crate per waktu.

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV: Rust 1.85](https://img.shields.io/badge/MSRV-Rust%201.85-blue.svg)](#minimum-supported-rust-version-msrv)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](#footprint)
[![GitHub stars](https://img.shields.io/github/stars/satyakwok/reliakit?style=flat)](https://github.com/satyakwok/reliakit/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/commits/main)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/satyakwok/reliakit)

Reliakit adalah workspace berisi crate kecil dan fokus untuk membangun software
Rust yang andal: CLI, service, bot, library, dan tool infrastruktur. Ide intinya
sederhana: **validasi dan batasi data di boundary, lalu bawa invariant yang
sudah terpercaya itu lebih dalam ke program kamu** supaya sisa kode tidak bisa
memegang state yang tidak valid.

Ini toolkit reliability serbaguna. Validated primitive, redaksi secret, bounded
collection, encoding deterministik, dan utilitas resilience yang runtime-agnostic
(retry backoff, circuit breaker, rate limiter, timeout) sama-sama berguna di
backend web, command-line tool, kode embedded, data pipeline, sampai kerjaan
protokol atau blockchain. Tidak ada satu pun dari itu yang jadi target utama.

Setiap crate kecil, bebas dependency saat runtime (cuma standard library dan
crate `reliakit-*` lain; ada cek CI yang menggagalkan build kalau ada dependency
pihak ketiga muncul), `#![forbid(unsafe_code)]`, dan bisa dipakai sendiri. Kamu
mengadopsi **satu crate per waktu**, bukan sebuah framework.

## Contoh 30 detik

Retry operasi yang labil pakai backoff, tanpa runtime, tanpa sleep, tanpa
dependency pihak ketiga:

```rust
use core::time::Duration;
use reliakit_retry::{retry, Backoff, RetryError, RetryPolicy};

let policy = RetryPolicy::new(3, Backoff::constant(Duration::from_millis(10))).unwrap();
let mut calls = 0;
let result: Result<u32, RetryError<&str>> = retry(
    &policy,
    || {
        calls += 1;
        if calls < 2 { Err("temporary") } else { Ok(42) }
    },
    |_error| true, // retry every error
);
assert_eq!(result.unwrap(), 42);
```

```toml
[dependencies]
reliakit-retry = "1"
```

## Apa bedanya

Reliakit itu **tanpa dependency pihak ketiga, `no_std`, dan runtime-agnostic**.
Tidak ada async runtime yang dipaksakan dan tidak ada yang sleep sendiri. Kamu
meng-inject clock atau sleeper, jadi kode yang sama bisa jalan secara sinkron, di
async runtime apa pun, atau di test tanpa waktu nyata. Kamu yang menyediakan
sumber waktu; sebagai gantinya kode tetap bebas dependency, portabel ke target
embedded, dan deterministik untuk dites.

## Kenapa Reliakit?

- **Validasi sekali, di boundary.** Buat nilai bertipe di tempat data masuk ke
  program (config, request, CLI, environment) dan jangan pernah cek ulang lagi.
- **Bikin state tidak valid susah direpresentasikan.** Sebuah `Port` selalu
  `1..=65535`; sebuah `BoundedStr<3, 32>` selalu punya 3–32 karakter. Signature
  tipenya mendokumentasikan sekaligus menegakkan aturannya untuk kamu.
- **Berhenti membocorkan secret.** Bungkus nilai sensitif dengan `Secret<T>` /
  `SecretString` supaya tampil sebagai `[REDACTED]` di `Debug`, `Display`, log,
  dan laporan error.
- **Batasi input dan collection kamu.** `BoundedVec<T, MIN, MAX>` tidak bisa
  dibuat di luar batas ukurannya.
- **Encode data secara deterministik.** `reliakit-codec` (binary) dan
  `reliakit-json` (text) menghasilkan byte yang sama untuk nilai yang sama,
  berguna untuk cache key, fixture, hashing, dan signing.
- **Tangani resilience secara eksplisit.** Backoff, circuit breaking, rate
  limiting, dan timeout adalah nilai biasa yang kamu kasih waktu sekarang, tanpa
  runtime, tanpa thread tersembunyi, tanpa global state.
- **Biaya adopsi tetap rendah.** Crate kecil yang independen compile cepat dan
  tidak menarik apa pun tambahan.
- **Satu keluarga yang menyatu: ambil satu atau semua.** Pakai satu crate untuk
  satu tugas, atau umbrella `reliakit` untuk beberapa; setiap blok mengikuti
  konvensi yang sama dan aturan yang sama: tanpa dependency, `no_std`, tanpa
  `unsafe`. Pola reliability biasanya berarti menyambung crate-crate yang tidak
  berhubungan dengan desain dan dependency tree berbeda; di sini semuanya memang
  dirancang agar pas.

## Footprint

Menambahkan Reliakit nyaris gratis; biaya yang biasanya kamu timbang sebelum
mengambil sebuah dependency kebanyakan tidak ada di sini:

- **Tanpa dependency pihak ketiga.** Dengan semua feature diaktifkan, seluruh
  dependency tree cuma crate `reliakit-*` dan standard library, tidak ada yang
  lain untuk ditelaah, diaudit, atau dipantau soal security advisory. Ada cek CI
  yang menggagalkan build kalau ada crate pihak ketiga muncul, dan
  `cargo tree -p reliakit --all-features` membuktikannya.
- **Tanpa `unsafe`.** Setiap crate mendeklarasikan `#![forbid(unsafe_code)]`.
- **Ramah `no_std`.** Crate inti bisa di-build untuk bare metal (misalnya
  `thumbv7em-none-eabi`); `alloc` dan `std` adalah feature opt-in.
- **Cold build cepat.** Tidak ada graph pihak ketiga untuk dicompile, jadi kamu
  membangun Reliakit saja, tidak ada yang lain.
- **Permukaan kecil dan mudah dibaca.** Setiap crate melakukan satu hal dan cukup
  kecil untuk dibaca dari awal sampai akhir sebelum kamu bergantung padanya.
- **Bayar hanya untuk yang kamu pakai.** Ambil satu crate, atau tarik beberapa
  lewat umbrella `reliakit` di balik feature flags per-crate.

## Fitur inti

| Area | Crate | Yang kamu dapat |
|---|---|---|
| Validated primitive | `reliakit-primitives` | `Port`, `Email`, `HttpUrl`, `Hostname`, `BoundedStr`, `Percent`, `SemVer`, `Uuid`, `HumanDuration`, … |
| Redaksi secret | `reliakit-secret` | `Secret<T>`, `SecretString`, `expose_secret` yang opt-in |
| Trait validation | `reliakit-validate` | Trait `Validate`, `ValidationError` yang mengumpulkan setiap pelanggaran field |
| Bounded collection | `reliakit-collections` | `BoundedVec<T, MIN, MAX>` dengan invariant ukuran yang ditegakkan |
| Codec binary kanonik | `reliakit-codec` | `CanonicalEncode` / `CanonicalDecode`, decoding yang ketat |
| JSON ketat | `reliakit-json` | Parser ketat + limit, output deterministik, `JsonEncode` / `JsonDecode` bertipe |
| CSV ketat | `reliakit-csv` | Reader ketat dan dibatasi + writer deterministik, `CsvEncode` / `CsvDecode` bertipe |
| Resilience | `reliakit-backoff`, `reliakit-bulkhead`, `reliakit-circuit`, `reliakit-ratelimit`, `reliakit-timeout` | Retry backoff, concurrency limiter, circuit breaker, token-bucket rate limiter, deadline, semuanya clock-agnostic |
| Helper retry | `reliakit-retry` | `RetryPolicy` + `retry` / `retry_with_sleep` / `retry_async`; runtime-agnostic, tidak pernah sleep secara internal |
| Pelaporan health | `reliakit-health` | Status `Health` + aggregator yang sadar-kritikalitas untuk `/health`, probe, dan halaman status |
| Clock bersama | `reliakit-core` | Trait `Clock` + `ManualClock` / `MonotonicClock` |
| Helper derive | `reliakit-derive` | `#[derive(CanonicalEncode, CanonicalDecode, JsonEncode, JsonDecode)]` |
| Logika keputusan | `reliakit-decide` | Keputusan deterministik berbasis utility (`Reasoner` dengan `decide`/`explain`/`gate`/`Policy`) |

## Blok resilience mana yang saya pakai?

Crate resilience masing-masing menyelesaikan satu masalah, dan masing-masing
adalah nilai biasa yang kamu jalankan dengan waktu sekarang, tanpa runtime, tanpa
thread tersembunyi, tanpa global state. Pilih berdasarkan pertanyaan yang kamu
ajukan:

| Pertanyaan | Blok | Crate |
|---|---|---|
| Berapa lama saya harus menunggu antar retry? | delay backoff + jitter | [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) |
| Retry panggilan yang bisa gagal dengan batas percobaan? | driver retry (sync + async) | [`reliakit-retry`](https://crates.io/crates/reliakit-retry) |
| Berhenti memanggil dependency yang terus gagal? | circuit breaker | [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit) |
| Batasi seberapa *sering* sesuatu boleh terjadi? | token-bucket rate limiter | [`reliakit-ratelimit`](https://crates.io/crates/reliakit-ratelimit) |
| Batasi *berapa banyak* yang jalan sekaligus, dan buang sisanya? | concurrency limiter (bulkhead) | [`reliakit-bulkhead`](https://crates.io/crates/reliakit-bulkhead) |
| Apakah budget waktu untuk operasi ini sudah habis? | deadline / timeout | [`reliakit-timeout`](https://crates.io/crates/reliakit-timeout) |

Mereka saling melengkapi, bukan tumpang tindih: `retry` menjalankan `backoff` di
antara percobaan; `circuit` berhenti memanggil dependency begitu cukup sering
gagal; `ratelimit` dan `bulkhead` membuang beban sebelum kamu mulai (terlalu
sering / terlalu banyak sekaligus); dan `timeout` membatasi keseluruhan operasi.
Tidak ada yang sleep atau spawn untukmu; kamu memasukkan clock (atau sleeper),
jadi semuanya tetap runtime-agnostic dan gampang dites.

Contoh [`resilient_client`](crates/reliakit/examples/resilient_client.rs)
menunjukkan timeout, rate limiter, circuit breaker, dan retry-with-backoff
bekerja sama dalam satu panggilan.

## Contoh penggunaan nyata

### 1. Validasi input backend / API

Validasi field request jadi nilai bertipe sekali saja, dekat dengan edge:

```rust
use reliakit_primitives::{Email, Port};

let contact = Email::new("ops@example.com")?;
let port = Port::new(8080)?;
assert_eq!(contact.domain(), "example.com");
assert_eq!(port.get(), 8080);
```

### 2. Tool CLI / parsing config + logging yang aman terhadap secret

Ubah config yang tipenya longgar jadi tipe terpercaya, dan jaga credential supaya
tidak masuk log:

```rust
use reliakit_primitives::{BoundedStr, Percent, Port};
use reliakit_secret::{ExposeSecret, SecretString};

type ServiceName = BoundedStr<3, 32>;

let name = ServiceName::new("api-service")?;
let success_rate = Percent::new(99)?;
let port = Port::new(8080)?;
let api_key = SecretString::from_string("rk_live_example");

assert_eq!(api_key.to_string(), "[REDACTED]"); // never leaks in Display/Debug/logs
assert_eq!(api_key.expose_secret(), "rk_live_example"); // explicit opt-in to read it
```

### 3. Microservices / panggilan eksternal: rate limiting dan circuit breaking

Nilai resilience yang clock-agnostic, kamu jalankan dengan sumber waktu sendiri:

```rust
use reliakit_ratelimit::RateLimiter;
use reliakit_circuit::{CircuitBreaker, State};

// Allow bursts of up to 10, refilling 1 token every 100 ms (~10/sec).
let mut limiter = RateLimiter::new(10, 1, 100);
assert!(limiter.try_acquire_one(0));

// Trip after 3 consecutive failures; stay open for 30_000 ms.
let mut breaker = CircuitBreaker::new(3, 30_000);
for _ in 0..3 {
    let _ = breaker.allow(0);
    breaker.on_failure(0);
}
assert_eq!(breaker.state(), State::Open); // fail fast instead of hammering a down service
```

### 4. Data pipeline / penanganan input yang dibatasi

```rust
use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
use reliakit_derive::{CanonicalDecode, CanonicalEncode};

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Record { id: u64, ok: bool }

let bytes = encode_to_vec(&Record { id: 7, ok: true })?;
assert_eq!(decode_from_slice_exact::<Record>(&bytes)?, Record { id: 7, ok: true });
```

### 5. JSON bertipe untuk API dan storage

```rust
use reliakit_derive::{JsonDecode, JsonEncode};
use reliakit_json::{from_json_str, to_json_string};

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Event { id: u64, name: String }

let json = to_json_string(&Event { id: 1, name: "deploy".into() });
assert_eq!(json, r#"{"id":1,"name":"deploy"}"#);
assert_eq!(from_json_str::<Event>(&json).unwrap(), Event { id: 1, name: "deploy".into() });
```

### 6. Constraint untuk embedded / ramah `no_std`

Crate resilience dan primitive yang bebas alokasi bekerja tanpa `std` atau bahkan
`alloc`. Sebuah `CircuitBreaker` atau `RateLimiter` adalah nilai `Copy` kecil
dengan aritmetika integer yang saturating dan bebas panic; kamu memasukkan tick
`u64`, jadi ia jalan di target embedded sama baiknya seperti di server.

### 7. Protokol dan encoding deterministik (termasuk blockchain)

Karena `reliakit-codec` mendefinisikan satu representasi byte kanonik per tipe dan
`reliakit-json` bisa menghasilkan JSON kanonik RFC 8785 (JCS) (feature `canonical`
yang opt-in), nilai yang sama selalu menghasilkan byte yang sama, berguna untuk
cache key, content addressing, dan hashing atau signing di kerjaan protokol dan
blockchain. Ini satu use case di antara banyak, bukan fokusnya.

### 8. Endpoint health dan readiness / halaman status

`reliakit-health` mengubah status per-komponen jadi satu jawaban untuk endpoint
`/health` atau `/readyz` atau halaman status. Kamu membangun `HealthReport` dari
check `critical` dan `optional`, dan agregatnya sadar-kritikalitas: dependency
`optional` (misalnya cache) yang `Unhealthy` menurunkan (degrade) service, bukan
menggagalkannya, sedangkan yang `critical` (database) menggagalkannya. Ia hanya
melaporkan; tidak pernah retry, sleep, atau bertindak.

### 9. Keputusan bertingkat yang bisa dijelaskan (routing, seleksi, agent)

`reliakit-decide` adalah decision engine deterministik kecil untuk saat
`if`/`else` terlalu kasar. Sebuah `Reasoner` memberi skor pada `Action` kandidat
dari `Consideration` berbobot yang dibentuk oleh sebuah `Curve`, dengan
`gate(...)` untuk hard constraint (opsi yang sedang down atau kena rate-limit
dilewati sepenuhnya) dan `explain()` untuk alasan kenapa sebuah pilihan menang,
berguna untuk request routing, memilih backend, atau menentukan kapan sebuah agent
harus memanggil LLM. Input sama, keputusan sama, setiap saat.

## Mulai cepat / instalasi

Cara tercepat masuk adalah lewat umbrella crate `reliakit`, yang me-re-export
setiap building block di balik sebuah feature flag. Tambahkan satu dependency dan
aktifkan hanya bagian yang kamu mau:

```toml
[dependencies]
reliakit = { version = "1.0", features = ["ratelimit", "secret"] }
```

```rust
use reliakit::ratelimit::RateLimiter;
use reliakit::secret::Secret;
```

Tidak ada yang ditarik di luar feature yang kamu aktifkan, jadi sifat
tanpa-dependency dan ramah `no_std` dari tiap blok tetap terjaga. Pakai
`features = ["full"]` untuk semuanya.

Mau dependency graph seketat mungkin? Crate-cratenya sepenuhnya independen; pakai
hanya yang kamu butuhkan:

```toml
[dependencies]
reliakit-primitives  = "1.0"
reliakit-secret      = "1.0"
reliakit-validate    = "1.0"
reliakit-collections = "1.0"
reliakit-codec       = "1.0"
reliakit-json        = "1.0"
reliakit-csv         = "1.0"
reliakit-backoff     = "1.0"
reliakit-retry       = "1.0"
reliakit-bulkhead    = "1.0"
reliakit-health      = "1.0"
reliakit-circuit     = "1.0"
reliakit-ratelimit   = "1.0"
reliakit-timeout     = "1.0"
reliakit-core        = "1.0"
reliakit-derive      = "1.0"
reliakit-decide      = "1.0"
```

Setiap crate independen; kebanyakan project memakai dua atau tiga. Minimum
supported Rust version adalah **1.85**.

## Ringkasan crate

| Crate | Tujuan | Pakai saat | Status |
|---|---|---|---|
| [`reliakit-primitives`](https://crates.io/crates/reliakit-primitives) | Tipe primitive yang tervalidasi | Kamu mau `Email`, `Port`, `Percent`, `BoundedStr`, … alih-alih string/angka yang tidak dicek. | Rilis (1.0) |
| [`reliakit-secret`](https://crates.io/crates/reliakit-secret) | Wrapper redaksi secret | Sebuah nilai tidak boleh bocor lewat `Debug`/`Display`/log. | Rilis (1.0) |
| [`reliakit-validate`](https://crates.io/crates/reliakit-validate) | Trait validation + agregasi error | Kamu mau mengumpulkan semua error field sekaligus. | Rilis (1.0) |
| [`reliakit-collections`](https://crates.io/crates/reliakit-collections) | Tipe bounded collection | Sebuah collection harus tetap dalam rentang ukuran tetap. | Rilis (1.0) |
| [`reliakit-codec`](https://crates.io/crates/reliakit-codec) | Encoding/decoding binary kanonik | Kamu butuh byte deterministik (cache key, fixture, framing). | Rilis (1.0) |
| [`reliakit-json`](https://crates.io/crates/reliakit-json) | JSON ketat dan deterministik + encode/decode bertipe | Kamu mem-parsing JSON yang tidak terpercaya atau butuh output yang bisa diprediksi. | Rilis (1.0) |
| [`reliakit-csv`](https://crates.io/crates/reliakit-csv) | CSV ketat dan deterministik + encode/decode bertipe | Kamu mem-parsing CSV yang tidak terpercaya atau butuh output yang reproducible. | Rilis (1.0) |
| [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) | Delay retry backoff + jitter | Kamu me-retry sebuah operasi dan mau spacing yang eksplisit. | Rilis (1.0) |
| [`reliakit-retry`](https://crates.io/crates/reliakit-retry) | Helper retry yang runtime-agnostic (sync + async) | Kamu me-retry operasi yang bisa gagal dan mau batas percobaan, backoff, dan classifier error tanpa memaksakan sebuah runtime. | Rilis (1.0) |
| [`reliakit-bulkhead`](https://crates.io/crates/reliakit-bulkhead) | Concurrency limiter (counting semaphore) | Kamu membatasi berapa operasi yang jalan sekaligus dan membuang sisanya. | Rilis (1.0) |
| [`reliakit-health`](https://crates.io/crates/reliakit-health) | Status health + aggregator yang sadar-kritikalitas | Kamu mengekspos endpoint `/health`/`readyz` atau halaman status. | Rilis (1.0) |
| [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit) | State machine circuit breaker | Kamu mau berhenti memanggil dependency yang gagal. | Rilis (1.0) |
| [`reliakit-ratelimit`](https://crates.io/crates/reliakit-ratelimit) | Token-bucket rate limiter | Kamu membatasi seberapa sering sesuatu boleh terjadi. | Rilis (1.0) |
| [`reliakit-timeout`](https://crates.io/crates/reliakit-timeout) | Deadline / budget waktu | Kamu melacak apakah sebuah budget sudah habis. | Rilis (1.0) |
| [`reliakit-core`](https://crates.io/crates/reliakit-core) | Trait `Clock` bersama + clock | Kamu mau sumber waktu `u64` siap pakai untuk crate resilience. | Rilis (1.0) |
| [`reliakit-derive`](https://crates.io/crates/reliakit-derive) | Macro derive untuk trait codec + JSON | Kamu mau `#[derive(...)]` daripada menulis encode/decode manual. | Rilis (1.0) |
| [`reliakit-decide`](https://crates.io/crates/reliakit-decide) | Decision engine deterministik berbasis utility | Kamu mau keputusan yang bertingkat, bisa dijelaskan, dan bisa dites (routing, seleksi, kapan memanggil LLM). | Rilis (1.0) |

Crate resilience (`backoff`, `bulkhead`, `circuit`, `ratelimit`, `timeout`)
bersifat **clock-agnostic**; kamu memasukkan waktu (di tempat yang
membutuhkannya), jadi mereka saling melengkapi dan bekerja di kode sync, async,
dan embedded: sebuah rate limiter memutuskan apakah memanggil, sebuah bulkhead
membatasi berapa panggilan yang jalan sekaligus, sebuah circuit breaker berhenti
memanggil dependency yang gagal, backoff memberi jarak antar retry, dan sebuah
timeout membatasi berapa lama kamu menunggu.

## Filosofi desain

- **Crate kecil dan independen** yang kamu adopsi satu per satu, tanpa lock-in
  framework.
- **Invariant eksplisit** yang divalidasi saat konstruksi; state tidak valid
  susah direpresentasikan.
- **API yang membosankan dan bisa diprediksi**: tipe dan trait biasa, tanpa
  runtime, thread, atau global state tersembunyi.
- **Tanpa runtime dependency** (cuma standard library + crate `reliakit-*` lain)
  dan `#![forbid(unsafe_code)]` di mana-mana.
- **Perilaku deterministik**: input sama, output sama; aritmetika saturating di
  crate resilience.
- **Integrasi yang di-gate feature**: tautan antar-crate (misalnya codec ↔
  primitives, JSON ↔ validate) adalah feature opt-in, tidak pernah default.

## Kapan memakai Reliakit

- Memvalidasi config, flag CLI, environment, atau payload request di boundary.
- Service backend, bot, dan library yang butuh constraint bertipe yang kecil.
- Menjaga secret supaya tidak masuk log dan diagnostik.
- Encoding deterministik untuk cache key, fixture, protokol, atau signing.
- Menambahkan logika retry/backoff/rate-limit/circuit-breaker/timeout yang
  eksplisit tanpa menarik async runtime.
- Kode embedded atau `no_std` yang butuh nilai yang dibatasi atau matematika
  resilience.

## Kapan tidak memakai Reliakit

Reliakit adalah sekumpulan building block kecil, bukan platform. Cari yang lain
kalau kamu butuh:

- web framework lengkap, HTTP stack, atau integrasi async runtime;
- ekosistem serialization lengkap dengan plugin format dan deserialization
  zero-copy;
- validation schema, tooling query/database, atau ORM;
- validator domain-specific di luar pengecekan Reliakit yang sengaja dibuat sempit
  (validation `Email`/`HttpUrl`-nya pragmatis, bukan implementasi RFC penuh).

## Feature flags & `no_std`

Reliakit ramah `no_std` di tempat yang masuk akal, tapi detailnya berbeda per
crate; cek README tiap crate untuk flag yang tepat.

- **Default feature** mengaktifkan `std`, yang otomatis menyertakan `alloc`. Build
  dengan `--no-default-features` memberi subset `no_std`.
- **API yang berbasis alokasi butuh `alloc`.** Tipe owned (berbasis `String`/`Vec`,
  misalnya `Email`, `BoundedStr`, `SecretString`, `BoundedVec`, semua isi
  `reliakit-json` dan `reliakit-csv`) memerlukan feature `alloc`; primitive yang
  bebas alokasi
  (`Port`, `Percent`, `Uuid`, `MacAddress`, `HumanDuration`, tipe numerik) jalan
  tanpa keduanya.
- **Crate resilience murni `core`.** `reliakit-backoff`, `reliakit-retry`,
  `reliakit-circuit`, `reliakit-ratelimit`, `reliakit-timeout`, dan
  `reliakit-core` tidak butuh alokasi sama sekali. `circuit`, `ratelimit`, dan
  `timeout` menyediakan feature `core` opsional yang menambahkan method praktis
  `*_now(clock)`. `reliakit-retry` tidak pernah sleep atau spawn; pemanggil yang
  meng-inject penungguan apa pun, jadi ia tidak memaksakan async runtime.
- **`reliakit-derive` adalah crate proc-macro.** Ia jalan saat compile time di
  host, jadi diskusi `no_std`/`alloc` yang biasa tidak berlaku untuknya; kode yang
  dihasilkannya mewarisi dukungan `no_std` dari crate trait-nya.

## Minimum Supported Rust Version (MSRV)

reliakit menargetkan **Rust 1.85**, minimum yang dibutuhkan oleh edition 2024.
MSRV dipatok di batas bawah ini, yang terendah yang diizinkan edition 2024, supaya
crate-cratenya tetap bisa dipakai sebagai dependency level rendah. Ini diverifikasi
di CI.

Menaikkan MSRV diperlakukan sebagai **breaking change**: ia dirilis dengan
kenaikan versi major dan dicatat di changelog. Ini tidak pernah dinaikkan
diam-diam di rilis patch, jadi mem-pin versi crate menjaga ia tetap bisa di-build
di Rust tempat ia dirilis.

## Kontribusi

Kontribusi dipersilakan. Baru di sini? [good first issue](https://github.com/satyakwok/reliakit/labels/good%20first%20issue)
yang dipin adalah tempat yang ramah untuk mulai, dan issue
[help wanted](https://github.com/satyakwok/reliakit/labels/help%20wanted) butuh
sedikit lebih banyak pertimbangan desain. Tolong buka issue dulu sebelum mengirim
pull request untuk perubahan yang non-trivial supaya arahnya bisa didiskusikan
dulu.

- Jaga tiap crate tetap minimal dan fokus.
- Tambahkan test untuk setiap permukaan API publik yang baru.
- Jalankan `cargo fmt`, `cargo clippy`, dan `cargo test` sebelum mengirim.

Lihat [`CONTRIBUTING.md`](./CONTRIBUTING.md) untuk panduan, [`CHANGELOG.md`](./CHANGELOG.md)
untuk catatan rilis, [`RELEASING.md`](./RELEASING.md) untuk proses rilis, dan
[`SECURITY.md`](./SECURITY.md) untuk pelaporan kerentanan.

## Star History

<a href="https://github.com/satyakwok/reliakit/stargazers">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&legend=top-left" />
 </picture>
</a>

## Lisensi

Dilisensikan di bawah MIT License. Lihat [`LICENSE`](./LICENSE).
