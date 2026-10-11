# Inventario de snapshots genuinos C2

25 archivos generados desde tipos/solver reales; detalles y campos ausentes en snapshot-inventory.json y new-snapshots.jsonl. Todo hash original verifica; todos restore/recaptura/round-trip/continuación ejercitados pasan. Leer no implica replay histórico exacto.

| Archivo | SHA256 estado original | Esquema/model original | Clase restaurada | Tiempo / combustible residual kg / pmax Pa |
|---|---|---|---|---|
| v1-cold.json | deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93 | 1 / None | exact_neutral | 0 / 0 / None |
| v1-pre.json | f30e11f748b6b2303f29f2bf3899b3ef440c93424641899f2028d4a077f1018d | 1 / None | approximate_hot | 0.20005 / 2.65488069e-05 / None |
| v1-mid.json | 3af5f58dd621ea27e07a5ceed451a979a46659a17b2d3a8845f1b866bc7abe7a | 1 / None | approximate_hot | 0.3596 / 5.28619077e-05 / None |
| v1-post.json | 99b49b9c2b1dca6c75e6f4834c63b37e8e49d6d5544ac4bf81032901c7c74d2b | 1 / None | approximate_hot | 0.38005 / 2.64919853e-05 / None |
| v1-hot.json | 7a6fe8a4f580e2e11c08ddfd380a3c7f7bb9c72e08c07f14b306d22687481fd7 | 1 / None | approximate_hot | 0.50005 / 2.10928404e-05 / None |
| s30-cold.json | 9c2bdb43cef1e8b74a4e14bf7036fd6249189520f0e8c9464ea5c843ad229561 | 2 / None | exact_neutral | 0 / 0 / None |
| s30-pre.json | a5bc6872b81a95dd80fa97a38ce9032f1e374b8c9f752274cac4a47d681cb5cc | 2 / None | approximate_hot | 0.20005 / 2.65488069e-05 / None |
| s30-mid.json | 3999d7a580a400803bc86556703ec12951a068e3395512f93f892877b72b676e | 2 / None | approximate_hot | 0.3596 / 5.28619077e-05 / None |
| s30-post.json | c705585cff16b9fe21e86789aa4138f09741df6a43e24418e3bf366f2428167a | 2 / None | approximate_hot | 0.38005 / 2.64919853e-05 / None |
| s30-hot.json | 5dfbc8221e5a0368ec684fc342e6a1292ad945e1d77dcaecb0730ef7a0369adc | 2 / None | approximate_hot | 0.50005 / 2.10928404e-05 / None |
| r02-cold.json | 9c2bdb43cef1e8b74a4e14bf7036fd6249189520f0e8c9464ea5c843ad229561 | 2 / None | exact_neutral | 0 / 0 / None |
| r02-pre.json | cc41975ffd27c2fc39b8673f9757b10e63ea0481ecdc3762cd2af33c2ef46b68 | 2 / None | approximate_hot | 0.20005 / 2.65488069e-05 / 419178.5043830399 |
| r02-mid.json | 151f2e8b54d7d154bedce8ae3473f3ba4d38831a468d23c40ae22a46b0f150b8 | 2 / None | approximate_hot | 0.3596 / 5.28619077e-05 / 5257195.751903657 |
| r02-post.json | eeb35289523bb7e2eca0fe1b9a0642b27158b70e2163e2af2e778d10e3626572 | 2 / None | approximate_hot | 0.38005 / 2.64919844e-05 / 5257195.751903657 |
| r02-hot.json | bd6ac6da6e18af2820660276dd018b48fb982d9c8a11c9bee94829a65507f2c5 | 2 / None | approximate_hot | 0.50005 / 2.07991143e-05 / 5257195.751903657 |
| c1-cold.json | 543f13d26e65db5cb110b29a3b3571b5bad3a703cce6474024bb0b79019968c9 | 2 / None | exact_neutral | 0 / 0 / 101325.0 |
| c1-pre.json | cc41975ffd27c2fc39b8673f9757b10e63ea0481ecdc3762cd2af33c2ef46b68 | 2 / None | approximate_hot | 0.20005 / 2.65488069e-05 / 419178.5043830399 |
| c1-mid.json | 5b6238349f4a6073138e7c5061d1b443a262cf86f3b2b815ce4880c9e03d8277 | 2 / None | approximate_hot | 0.3596 / 2.65630125e-05 / 5257195.751903657 |
| c1-post.json | 550d883a319f37e0fa9a49d742656af1fc7213dbb9fe4e34a825d17718208199 | 2 / None | approximate_hot | 0.38005 / 2.64944043e-05 / 5257195.751903657 |
| c1-hot.json | 363a48357fe1034dcca9a863601763ed84d8e175e1b39787c50d01f6bbc17f51 | 2 / None | approximate_hot | 0.50005 / 2.0798306e-05 / 5257195.751903657 |
| c2-cold.json | 9c2bdb43cef1e8b74a4e14bf7036fd6249189520f0e8c9464ea5c843ad229561 | 2 / {'combustion_accounting': 1, 'schema_version': 2} | C2 nativo sin migración | 0 / 0 / None |
| c2-pre.json | cc41975ffd27c2fc39b8673f9757b10e63ea0481ecdc3762cd2af33c2ef46b68 | 2 / {'combustion_accounting': 1, 'schema_version': 2} | C2 nativo sin migración | 0.20005 / 2.65488069e-05 / 419178.5043830399 |
| c2-mid.json | 5b6238349f4a6073138e7c5061d1b443a262cf86f3b2b815ce4880c9e03d8277 | 2 / {'combustion_accounting': 1, 'schema_version': 2} | C2 nativo sin migración | 0.3596 / 2.65630125e-05 / 5257195.751903657 |
| c2-post.json | 550d883a319f37e0fa9a49d742656af1fc7213dbb9fe4e34a825d17718208199 | 2 / {'combustion_accounting': 1, 'schema_version': 2} | C2 nativo sin migración | 0.38005 / 2.64944043e-05 / 5257195.751903657 |
| c2-hot.json | 363a48357fe1034dcca9a863601763ed84d8e175e1b39787c50d01f6bbc17f51 | 2 / {'combustion_accounting': 1, 'schema_version': 2} | C2 nativo sin migración | 0.50005 / 2.0798306e-05 / 5257195.751903657 |

Productores:
- 254126618fb17b5e09c398078b2f1efd1963de25
- ff09a0dda82188e6bf27a80d17c902f8972a39d2
- 8a7e8686e4e8831fb30d1a71b815e67e7d7139ef
- caad1cbf31d01531e0fbd81be6190f52b2d1c2d7
- a0040c58174a6cf6a2ac84e38605eafd2f584159

Pre/mid/post son tiempos solicitados0.2/0.35955/0.38s; el solver histórico puede sobrepasar un dt. Las banderas burn_accounted completas y el inventario real quedan en JSON: no se impone que una misma hora represente idéntica fase entre modelos. Hot0.5s incluye combustible residual y memoria de presión donde el esquema la tiene. No existe captura térmica v1; esa ausencia se registra, no se inventa.
