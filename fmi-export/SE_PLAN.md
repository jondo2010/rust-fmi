# Implementar Scheduled Execution (SE) en `fmi-export` — análisis y plan

> Documento de trabajo. Estado del código analizado: rama local en
> `Estancia UPM/rust-fmi` (commit del 30-jun-2026). Todos los números de línea
> de este documento se verificaron contra ese árbol.

## TL;DR

`fmi-export` tiene la **fachada** de Scheduled Execution completa (declara el
flag, exporta los símbolos C, soporta `intervalVariability="Countdown"` en la
macro), pero el **motor** son *stubs*: `activate_model_partition` devuelve
siempre `Error` y las funciones de intervalo de reloj son `todo!()` (pánico a
través de FFI = comportamiento indefinido). Por eso el FMU del semáforo
(`rust_fmus/semaforo_se`) hubo que escribirlo con el ABI FMI a mano.

Hay que tocar **tres sitios** para SE periódico, **dos más** para relojes
*countdown* (aperiódicos), y hay **dos decisiones de diseño** que giran en
torno al trait `UserModel` (esto es lo que menciona el tutor).

---

## 1. El pastel de capas (imprescindible antes de tocar nada)

Es fácil confundir `UserModel` con una macro. No lo es. Conviven **dos macros**
y **varios traits**:

| Nombre | Qué es | Quién lo escribe |
|--------|--------|------------------|
| `#[derive(FmuModel)]` | **macro** derive | el usuario, sobre su struct |
| `export_fmu!(MiModelo)` | **macro** de función | el usuario, una vez |
| `UserModel` | **trait**: la física del modelo | **el usuario, a mano** |
| `Model` | trait: metadatos (nombre, nº estados, flags ME/CS/SE) | lo genera el derive |
| `ScheduledExecution` / `ModelExchange` / `CoSimulation` | traits: semántica FMI de cada interfaz | `fmi-export`, sobre `ModelInstance` |
| `Fmi3Common` / `Fmi3ScheduledExecution` / … | traits *wrapper*: ABI C crudo | `fmi-export`, con impl por defecto |

### Flujo de una llamada (activar una partición)

```
1. fmi3ActivateModelPartition(inst, clock_ref, t)              ← extern "C", generada por export_fmu!
2.   <MiModelo as Fmi3ScheduledExecution>::fmi3_activate_model_partition(...)   ← wrapper ABI (wrappers.rs)
3.     <ModelInstance<MiModelo> as ScheduledExecution>::activate_model_partition(...)  ← STUB (impl_se.rs)
4.       self.model.<algún método de UserModel>(...)           ← la física del usuario
```

- El **paso 2 ya está hecho** ([wrappers.rs:1389](src/fmi3/traits/wrappers.rs)):
  `fmi3_activate_model_partition` ya hace el *dispatch* correcto.
- El **paso 3 es el stub** que hay que rellenar.
- El **paso 4 no existe todavía**: `UserModel` no tiene ningún método pensado
  para "activar una partición" ni para "declarar el próximo intervalo". Ese es
  el hueco de diseño.

### Qué es `activate_model_partition`

En SE el modelo se divide en **particiones**, cada una atada a un *input clock*.
El scheduler externo llama a `fmi3ActivateModelPartition(clock_reference,
activation_time)` cuando le toca el turno a ese reloj = "ejecuta un tick de esta
partición ahora". Argumentos:

- `clock_reference`: VR del reloj → qué partición activar.
- `activation_time`: tiempo virtual del tick.

**`activate_model_partition` es a SE lo que `do_step` es a CS.** La
implementación de `do_step` en [impl_cs.rs:57](src/fmi3/instance/impl_cs.rs) es
casi la plantilla exacta.

---

## 2. Hechos ya verificados del código (no hay que rehacerlos)

- ✅ `ModelState::ClockActivationMode` **existe** ([mod.rs:59](src/fmi3/mod.rs)).
- ✅ Al salir de inicialización, una instancia SE **ya pasa** a
  `ClockActivationMode` ([common.rs:101-103](src/fmi3/instance/common.rs)); ídem
  al salir de `ReconfigurationMode` ([common.rs:173-175](src/fmi3/instance/common.rs)).
- ✅ El wrapper ABI `fmi3_activate_model_partition` ya despacha al método de
  instancia ([wrappers.rs:1389](src/fmi3/traits/wrappers.rs)).
- ✅ `export_fmu!` ya exporta todos los símbolos C de reloj
  ([export.rs:565-665](src/fmi3/export.rs)).

**Conclusión de diseño nº1 (cerrada):** `activate_model_partition` NO necesita
un estado nuevo. Debe exigir `self.state == ModelState::ClockActivationMode`
(igual que `do_step` exige `StepMode`).

---

## 3. Mapa de trabajo

Tres bloques bien diferenciados. **No mezclarlos.**

### Bloque A — Núcleo mínimo (SE periódico funcionando)

**A.1 — Implementar `activate_model_partition`**
Archivo: [`src/fmi3/instance/impl_se.rs`](src/fmi3/instance/impl_se.rs) (hoy: log + `Err`).
Plantilla (calcada de `do_step`):

```rust
fn activate_model_partition(
    &mut self,
    clock_reference: fmi::fmi3::binding::fmi3ValueReference,
    activation_time: f64,
) -> Result<Fmi3Res, Fmi3Error> {
    self.context.log(
        Fmi3Res::OK.into(),
        M::LoggingCategory::trace_category(),
        format_args!("activate_model_partition(clk: {clock_reference}, t: {activation_time})"),
    );
    self.assert_instance_type(fmi::InterfaceType::ScheduledExecution)?;

    // 1) estado válido
    match self.state {
        ModelState::ClockActivationMode => {}
        _ => {
            self.context.log(/* error */);
            return Err(Fmi3Error::Error);
        }
    }

    // 2) fijar el tiempo del tick
    self.context.set_time(activation_time);

    // 3) ejecutar la física de ESTA partición  ← ver decisión de diseño nº2
    self.model.activate_partition(&mut self.context, clock_reference, activation_time)?;

    // 4) marcar salidas sucias para que los getters recalculen
    self.is_dirty_values = true;
    Ok(Fmi3Res::OK)
}
```

**A.2 — Decisión de diseño nº2 (a acordar con el tutor): el hook en `UserModel`.**
Archivo: [`src/fmi3/traits/mod.rs:185`](src/fmi3/traits/mod.rs).
`UserModel` hoy tiene `configurate`, `calculate_values`, `event_update`,
`get_event_indicators`, `do_step` — pero nada de particiones. Opciones:

- **(a)** Reusar `calculate_values` directamente. Rápido, pero no distingue qué
  reloj se activó → insuficiente para multi-partición.
- **(b) [recomendada]** Añadir un método con impl por defecto:

  ```rust
  /// Activa la partición ligada al reloj `clock_reference` en el instante `time`.
  /// Por defecto recalcula todo el modelo (válido para 1 sola partición).
  fn activate_partition(
      &mut self,
      context: &mut dyn Context<Self>,
      _clock_reference: fmi::fmi3::binding::fmi3ValueReference,
      _time: f64,
  ) -> Result<Fmi3Res, Fmi3Error> {
      self.calculate_values(context).map(|_| Fmi3Res::OK)
  }
  ```

  Ventaja: modelos simples no escriben nada; modelos multi-reloj lo sobrescriben
  y hacen `match clock_reference { … }`. **Esto es lo que insinúa el tutor con
  "algo de UserModel".**

### Bloque B — Countdown clocks (aperiódicos, el caso del semáforo)

**B.1 — Implementar `fmi3_get_interval_decimal` / `_fraction`**
Archivo: [`src/fmi3/traits/wrappers.rs:657` y `:668`](src/fmi3/traits/wrappers.rs) (hoy: `todo!()`).

Un *countdown clock* es exactamente esto: tras cada activación, el modelo
declara **cuánto falta para el siguiente tick** (σ, el `ta()` de DEVS). El
importer lo lee con `fmi3GetIntervalDecimal`.

**Decisión de diseño nº3:** hoy estos wrappers hacen `todo!()` *directo*, sin
delegar a ningún trait (a diferencia de `activate`, que sí despacha). En el
crate core `fmi`, el trait `Common` tiene `get_clock`/`set_clock` pero **NO**
`get_interval_*` ([`fmi/src/fmi3/traits.rs:111`](../fmi/src/fmi3/traits.rs)). Por
tanto hay que **crear el método de nivel superior** al que delegar. Encaja
natural en `UserModel`:

```rust
/// Devuelve el próximo intervalo (segundos) del reloj `clock_reference`,
/// o `None` si el intervalo no ha cambiado (fmi3IntervalUnchanged).
fn next_interval(&self, _clock_reference: fmi3ValueReference) -> Option<f64> {
    None
}
```

Y el wrapper: leer VRs, para cada uno llamar `next_interval`, escribir en
`intervals[]` y poner el `qualifier` (`fmi3IntervalChanged` / `Unchanged`).
`_fraction` = lo mismo pero como racional (contador/resolución).

### Bloque C — Resto de TODOs de reloj (frontera del alcance)

Siguen siendo `todo!()` en [`wrappers.rs:682-748`](src/fmi3/traits/wrappers.rs)
y `:751`:
`get_shift_decimal/fraction`, `set_interval_decimal/fraction`,
`set_shift_decimal/fraction`, `evaluate_discrete_states`.

> **Nota:** `update_discrete_states` ([wrappers.rs:758](src/fmi3/traits/wrappers.rs))
> **ya está implementado** (despacha a `update_discrete_states` de la instancia en
> [common.rs:205](src/fmi3/instance/common.rs) → `event_update` de `UserModel`).
> No es un stub.

El obstáculo NO es dificultad, son tres cosas concretas:

1. **No se pueden validar**: ningún modelo actual (los 7 de MathCore ni el
   semáforo) ejercita un reloj *tunable* (intervalo fijado por el entorno) ni un
   *shift*. Implementar `set_interval`/`set_shift` sin un modelo que los use
   significa enviar código FFI `unsafe` **sin test posible** → solo "validado"
   leyendo el spec. Ahí es donde se esconden las malinterpretaciones sutiles del
   estándar. Un `fmi3Error` honesto es **más correcto** que una implementación
   plausible-pero-no-verificada.
2. **Falta diseño de almacenamiento**: operan sobre intervalo/shift *por reloj*,
   que hoy no tienen sitio en `ModelInstance`. Diseñar esa tabla antes de que
   exista un modelo multi-reloj es prematuro (YAGNI).
3. **No están en la ruta crítica**: ni el periódico-fijo (intervalo en el XML) ni
   el countdown (el FMU *declara* con GET) necesitan las funciones SET/shift.
   Implementarlas no habilita nada que podamos usar hoy.

**PERO — punto crítico de seguridad, obligatorio igualmente:** mientras sean
`todo!()`, si el importer las llama la FMU **entra en pánico a través de FFI =
UB**. `fmi3EvaluateDiscreteStates` se exporta ([export.rs:699](src/fmi3/export.rs)),
así que es alcanzable. Aunque no se implementen, hay que **cambiar `todo!()` por
`log(...) + fmi3Error`**. Eso convierte "fuera de alcance" en algo *seguro*, y es
una mejora legítima por sí sola (Fase 0).

**Subconjunto barato y seguro (opcional, se puede hacer ya):**
`get_shift_decimal/fraction` devolviendo el shift declarado (0.0 si no hay) y
`evaluate_discrete_states` como passthrough a `calculate_values`. Las **SET** son
las que conviene dejar en `fmi3Error` hasta tener un modelo que las ejercite.

### Fuera de alcance — el crate `fmi` (importador)

`fmi/src/fmi3/instance/scheduled_execution.rs` tiene sus propios `todo!()`
(`clock_update`, `lock_preemption`…). Eso es el lado **importador/simulador**
(para que `fmi-sim` *ejecute* FMUs SE). Esta tarea es el lado **exportador**
(`fmi-export`). No tocar salvo que además se quiera simular SE con `fmi-sim`.

---

## 4. Plan por fases

```
Fase 0 — Seguridad (trivial):   todo!() → log + Fmi3Error en los 9 wrappers de reloj/discrete que siguen en todo!()
Fase 1 — Núcleo:                A.1 activate_model_partition + A.2 hook activate_partition en UserModel
Fase 2 — Aperiódico:            B.1 get_interval_decimal delegando a next_interval() de UserModel
Fase 3 — Validación:            exportar semaforo_se con fmi-export y comparar bit a bit con el ABI a mano
```

**Criterio de "hecho" de la Fase 3:** el FMU del semáforo generado por
`fmi-export` debe producir la misma traza que el `semaforo_se` escrito a mano
(rojo 60 s / verde 30 s, y rojo→15 s al pulsar el botón), corriendo en el
orquestador `SeRo_CoSim`.

---

## 5. Decisiones a cerrar con el tutor (resumen)

1. **[CERRADA]** ¿Hace falta un `ModelState` nuevo para SE? → **No**,
   `ClockActivationMode` ya existe y ya se entra en él.
2. **[ABIERTA]** ¿`activate_model_partition` reusa `calculate_values` o
   añadimos `UserModel::activate_partition` (con default → `calculate_values`)?
   → Propuesta: **añadir el método** (opción b).
3. **[ABIERTA]** ¿El intervalo de los countdown clocks se expone como
   `UserModel::next_interval(clock_vr) -> Option<f64>`? → Propuesta: **sí**; es
   la traducción natural del `ta()` de DEVS y lo que ya hace a mano el semáforo.

---

## 6. Índice de archivos tocados

| Archivo | Bloque | Acción |
|---|---|---|
| `src/fmi3/instance/impl_se.rs` | A.1 | Implementar `activate_model_partition` |
| `src/fmi3/traits/mod.rs` (`UserModel`) | A.2, B.1 | Añadir `activate_partition` y `next_interval` (con default) |
| `src/fmi3/traits/wrappers.rs` (`Fmi3Common`) | B.1 | Implementar `fmi3_get_interval_decimal/_fraction` |
| `src/fmi3/traits/wrappers.rs` (`Fmi3Common`) | 0 / C | `todo!()` → log + `fmi3Error` en shift/set/discrete |
| `rust_fmus/semaforo_se` (en SeRo_CoSim) | 3 | Reescribir con `#[derive(FmuModel)]` + `export_fmu!` para validar |

### Referencia rápida de los stubs actuales

- `impl_se.rs:10` → `activate_model_partition` devuelve `Err` siempre.
- `wrappers.rs:657` → `fmi3_get_interval_decimal` = `todo!()`.
- `wrappers.rs:668` → `fmi3_get_interval_fraction` = `todo!()`.
- `wrappers.rs:682,692` → `get_shift_decimal/fraction` = `todo!()`.
- `wrappers.rs:704,715` → `set_interval_decimal/fraction` = `todo!()`.
- `wrappers.rs:727,738` → `set_shift_decimal/fraction` = `todo!()`.
- `wrappers.rs:751` → `evaluate_discrete_states` = `todo!()`.
- `wrappers.rs:758` → `update_discrete_states` = **YA implementado** (no es stub).
