# Motor integrado end-to-end — S16

## Acoplo

Bucle temporal: flujos S12 (orificios + Cd + nodos 2 especies) por
cilindro y fase S11, inyección S13 agrupada al IC con rail del plenum,
chispa S15 al cruce del avance, quemado S10 por reloj local, cigüeñal
S14 (τ de presiones vivas + fricción + starter S15 + accesorios).
Control open-loop (ηv 0.75, AFR 13.5, avance 15° fijos, declarados).

## Verificación (ilustrativo, 2026-10-10)

Punto ralentí (mariposa 0.05 + bypass 0.02 cm²): ~1026 rpm estables a
8 s; mariposa mayor ⇒ 1920/3102/5552 rpm (speed-density emergente).
no_spark/no_fuel ⇒ 0 rpm; fricción ×3 ⇒ 699 rpm; corte en marcha ⇒
decae. Masa ~1e−11/1e−14; energía 7e−5 (sano) / 1e−3 (arrastre).
Convergencia dt < 10%; snapshot bit-idéntico; determinismo total.

## Dominio y deuda

Sin lazo cerrado, sin AFR→quemado, sin multicilindro diferenciado,
sin periódico convergido multiciclo más allá de 8 s, sin arranque en
caliente/frío parametrizado, sin control de ralentí (S19), sin
diagnóstico (S20). Régimen reportado tal cual (gate S16).
