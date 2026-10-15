# Contrato de snapshots — S30-T6 (+ remediación R02 y R02-C1, 2026-10-10)

- Esquema engine v2 (`schema_version`), wear v1.
- JSON canónico (`engine-types::canonical_json`, claves ordenadas) +
  SHA-256 (`state_hash`).
- **Verificación antes de migrar (R02-F06):** `restore` reconstruye la
  representación canónica *del esquema declarado* y verifica el hash
  **antes** de aplicar cualquier migración. El v1 histórico (anterior a S21)
  no tenía campo `thermal`: su hash se comprueba sobre los bytes v1 y luego
  se migra a v2. Un hash v1 válido ya no se rechaza por añadir `thermal:null`.
- **Compatibilidad (R02-F05):** la ruta de parseo de f64 es lossless
  (`serde_json/float_roundtrip`); el round-trip por disco preserva bits y
  hash. Se separa integridad de archivo (hash) de validez física del estado.
- **Campos opcionales de R01 (R02):** `plen_fuel_kg`, `exh_fuel_kg`,
  `fuel_emitted_kg` y `pmax_pa` se omiten de la forma canónica cuando valen su
  valor neutro (cero **positivo** o ausente). Los snapshots v1/v2 previos
  conservan su hash histórico; los nuevos serializan el estado físico completo
  (especies en nodos + historia de presión). No hay bump de esquema: la
  extensión es retrocompatible y se lee con `#[serde(default)]`.
- **Cero firmado (R02-C1/RV03):** el cero negativo (`-0.0`) es un valor
  representable distinto y **conserva su bit de signo** al serializar/parsear
  (solo se omite `+0.0`). La garantía de identidad de bits de estados válidos
  aplica a ambos signos; los snapshots previos sin el campo llegan como `+0.0`
  y no cambian de hash.
- **Procedencia de migración (R02-C1/RV04):** un snapshot legacy migrado lleva
  `provenance` (esquema y hash de origen, transformación, campos históricos
  ausentes y alcance). `continuation` es `exact_neutral` (estado inicial sin
  historia) o `approximate_hot` (había combustible/movimiento: la trayectoria
  histórica no se reconstruye). Los snapshots nativos completos no llevan
  procedencia y mantienen continuidad exacta.
- **Identidad semántica de modelo (R02-C2/C1-05 + R02-C3/C2-04):** `model`
  (en el wrapper) separa la versión del esquema de serialización de la
  **convención de contabilidad química** (`combustion_accounting`). Un snapshot
  sin `model` (formato pre-C2) es ambiguo entre R02 (consumo diferido) y C1
  (por paso): se migra con procedencia `approximate_hot`/`exact_neutral` y
  **nunca** se declara nativo exacto. Un `model` presente con esquema
  desconocido (∉ {1,2}) o contabilidad desconocida (≠ 1) se **rechaza** en
  todos los esquemas, incluido v1, aunque el hash sea válido. **C4:** además,
  todo marcador presente debe cumplir `model.schema_version ==
  state.schema_version`; no hay excepciones frías ni por fixture. El v1
  genuino sin `model` conserva su lectura. El responsable autorizó adaptar
  únicamente una copia derivada del fixture reetiquetado a v1, estableciendo
  `model=None`; los oráculos y su ejecución literal permanecen intactos y
  reportados por separado (ADR-042 y reporte C4).
- **Historia de presión requerida (R02-C3/C2-05):** un snapshot con `model`
  conocido y esquema actual solo se declara **nativo exacto** si `pmax_pa` está
  presente **cuando la fricción activa depende de presión** (Chen–Flynn
  `c2 ≠ 0`). Si falta `pmax_pa` con `c2 ≠ 0`, la continuación se etiqueta
  `approximate_hot`; con `c2 = 0` la falta no altera la trayectoria y puede
  continuar exacta. No se reconstruye presión histórica.
- **Alcance de integridad (C1-G01):** `state_hash` autentica **solo el estado
  canónico**. `model` y `provenance` son metadata **no autenticada** por el
  hash: un consumidor que necesite confiar en la procedencia debe verificarla
  contra una fuente externa fiable; reserializar no convierte la metadata en
  autenticada. No se firma criptográficamente el archivo ni se reescriben
  hashes históricos.
- **Validez física fail-closed (R02-F15, R02-C1/RV05–RV06):** un hash válido no
  implica estado válido. `restore` rechaza masas/temperaturas no positivas o no
  finitas, fracciones fuera de [0,1], combustible fuera de [0, m], historia de
  eventos no finita, contadores negativos/no finitos, estados térmicos bajo el
  cero absoluto y esquemas desconocidos; valida también los inputs y nunca
  repara en silencio.
- **Replay (R02-F07):** se persiste toda variable con memoria física que
  afecte la trayectoria, incluido `pmax_pa` (máximo de presión de cilindro que
  alimenta el término de presión de Chen–Flynn). El replay continuo y el
  dividido en cualquier corte coinciden bit a bit para `c2 ≠ 0`.
- Compatibilidad declarada: v2 lee v1 (migración verificada, continuación
  legacy etiquetada). Cambios de esquema incompatibles exigen bump + migración
  testeada + entrada en CHANGELOG; esta extensión es compatible, no la exige.
- `docs/export-contract.md` (`trace-bundle-v1`) sigue vigente para el
  spike visual: solo lectura, sin Blender/UI en el core.

## Composición de procedencia y fronteras C4 (2026-10-15)

- Ante una nueva pérdida relevante de historia, `provenance` describe el
  snapshot entrante actual (esquema/hash, transformación, campos ausentes y
  alcance). `prior`, opcional y vacío por defecto, conserva el registro
  anterior **completo**. Cada eslabón conserva a su vez su historia previa.
  No se sobrescribe el origen ni se inventa la presión perdida.
- Un origen `exact_neutral` no oculta una nueva continuación caliente
  aproximada. Una limitación previa `approximate_hot` no se eleva a exacta
  por recapturar defaults. Releer el mismo envelope ya registrado no agrega
  otro eslabón; una recaptura nativa completa conserva la limitación.
- `prior` pertenece al wrapper, fuera de `state_hash`, igual que el resto
  de la procedencia. Ausente se lee como lista vacía y vacío se omite al
  escribir; no cambian la forma canónica del estado, los hashes históricos,
  los bits de f64 ni los campos físicos/ModelIdentity. El struct Rust de
  metadata añade un miembro público; callers que lo construyan por literal
  deben inicializar `prior: Vec::new()`. La compatibilidad de lectura JSON
  no implica compatibilidad de todos los literales de source externos.
- La presión de plenum/escape debe ser finita y positiva, igual que su
  densidad, antes de aceptar restore o resume(incluido avance nulo).
  Overflow/underflow se rechazan sin cotas OEM arbitrarias.
- La resistencia calculada de batería devuelve error tipado si no es
  representable, antes de limitar el terminal; temperaturas Celsius
  negativas legítimas y resistencias ideales cero siguen en su dominio.

Estado de la implementación C4: **IMPLEMENTED_PENDING_INDEPENDENT_REVIEW**.
No es un gate de integración ni validación experimental.
