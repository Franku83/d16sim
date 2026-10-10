# THIRD_PARTY_NOTICES — S02

Generado 2026-10-10 desde `Cargo.lock` (44 paquetes) cruzado con
`~/.cargo/registry/src/*/…/Cargo.toml` (`license = …`). Método reproducible:
ver comando en `docs/sprints/S02.md`. Código propio: Apache-2.0 (`LICENSE`).

**Resultado auditoría S02: todas las dependencias son permisivas
(MIT / Apache-2.0 / Unlicense / Unicode-3.0). Ninguna GPL/copyleft.**
Speeduino (GPL-2.0) y manual Honda siguen siendo solo referencias, no
dependencias. OpenWAM: sin integrar (licencia por auditar).

| Paquete | Versión | Licencia declarada upstream |
|---|---|---|
| anstream | 1.0.0 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 (solo Windows; no compilado en Linux) |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| clap / clap_builder / clap_derive / clap_lex | 4.6.7 / 1.1.1 | MIT OR Apache-2.0 |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| generic-array | 0.14.7 | MIT |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 (solo Windows; no compilado en Linux) |
| proc-macro2 / quote / syn / unicode-ident | 1.0.107 / 1.0.47 / 3.0.6 / 1.0.26 | MIT OR Apache-2.0 (unicode-ident + Unicode-3.0) |
| serde / serde_core / serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| strsim | 0.11.1 | MIT |
| thiserror / thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| windows-link / windows-sys | 0.2.1 / 0.61.2 | MIT OR Apache-2.0 (solo Windows; no compilados en Linux) |
| zmij | 1.0.23 | MIT |

Las 4 entradas solo-Windows se verifican en CI Windows (`rust-ci.yml`).
`cargo-deny`: evaluado; no adoptado en S02 (auditoría manual sobre
`Cargo.lock` documentada aquí; reevaluar si el árbol supera ~200 paquetes).
