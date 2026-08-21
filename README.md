# Lumen

**Procesador de textos para escritores en terminal Linux.**

Filosofía: *el poder está disponible cuando se necesita, nunca compite con el acto de escribir. El escritor escribe. Lumen recuerda.*

```
Lumen v0.1.0
```

---

## Índice

- [Descripción](#descripción)
- [Características](#características)
- [Requisitos](#requisitos)
- [Instalación](#instalación)
- [Uso rápido](#uso-rápido)
- [Atajos de teclado](#atajos-de-teclado)
- [Proyectos](#proyectos)
- [Panel Creativo](#panel-creativo)
- [Ortografía](#ortografía)
- [Formatos de archivo](#formatos-de-archivo)
- [Configuración](#configuración)
- [Estructura del código](#estructura-del-código)
- [Licencia](#licencia)

---

## Descripción

Lumen es un procesador de textos minimalista diseñado para escritores que trabajan en terminal. Está pensado para Linux, pero puede compilarse en otros sistemas para desarrollo.

**Lo que Lumen es:**
- Un editor de texto ligero y rápido para la terminal
- Un asistente de escritura creativa (personajes, lugares, timeline, conceptos)
- Un corrector ortográfico integrado
- Un gestor de proyectos literarios con capítulos

**Lo que Lumen no es:**
- No es un IDE
- No es un procesador de textos con formato WYSIWYG
- No necesita internet ni inteligencia artificial
- No almacena datos en la nube

Toda la información se guarda en formatos TOML abiertos y portables. Puedes mover tu proyecto a cualquier computadora y funciona sin cambios.

---

## Características

### Fase 1 — Editor base
- Edición de texto con cursor
- Deshacer / rehacer (árbol completo)
- Buscar y reemplazar
- Ir a línea
- Seleccionar texto (shift + flechas, palabra, página)
- Copiar / cortar / pegar
- Numeración de líneas
- Estado: posición del cursor, palabras, líneas

### Fase 2 — Paneles laterales
- **Notas** (F2): notas rápidas con separador `---`
- **Ideas** (F4): ideas sueltas para desarrollo
- Navegación con ↑↓, crear (N), editar (Enter), borrar (D)

### Fase 3 — Ortografía
- Corrector ortográfico integrado con `spellbook` (Hunspell)
- Detección automática de idioma
- Panel de errores (F3) con sugerencias
- Diccionario personal (agregar palabras ignoradas)
- Subrayado visual de errores en el texto
- Cambio de idioma en vivo

### Fase 4 — Documentos y proyectos
- **Formato TXT**: formato nativo, guardado atómico
- **Formato ODT**: apertura y exportación de archivos OpenDocument (libreoffice)
- **Sistema de proyectos**: estructura de directorios con `.lumen/project.toml`
- Capítulos con IDs estables (resisten reorden y eliminación)
- Estados de capítulo: Borrador → En revisión → Revisado → Finalizado
- Autoguardado cada 30 segundos
- Recuperación de archivos de respaldo

### Fase 5 — Contexto creativo
- **Personajes**: nombre, descripción, notas, asociaciones con capítulos
- **Lugares**: nombre, descripción, notas, asociaciones con capítulos
- **Conceptos**: nombre, descripción, notas
- **Línea de tiempo**: eventos ordenados cronológicamente, asociaciones con capítulos
- **Estadísticas**: palabras, capítulos, promedio, porcentaje de finalización
- Persistencia en `.lumen/creative.toml`
- Panel unificado (F5) con secciones navegables

---

## Requisitos

### Sistema operativo
Linux (diseñado para). Puede compilarse en Windows/macOS con la variable de entorno:

```bash
export LUMEN_ANY_OS=1
```

### Rust
- Rust 2021 edition (1.56+)
- `cargo` (gestor de paquetes)

Para instalar Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Dependencias del sistema (Linux)

En Ubuntu/Debian:
```bash
sudo apt install build-essential pkg-config libssl-dev
```

En Arch:
```bash
# Rust y cargo ya vienen con rustup
```

Las dependencias de Rust se instalan automáticamente con `cargo build`.

---

## Instalación

### Desde GitHub

```bash
# Clonar el repositorio
git clone https://github.com/TU_USUARIO/lumen.git
cd lumen

# Compilar en modo release (optimizado)
cargo build --release

# El ejecutable queda en:
# target/release/lumen
```

### Instalar globalmente

```bash
# Opción 1: copiar el ejecutable manualmente
cp target/release/lumen ~/.local/bin/

# Opción 2: usar cargo install
cargo install --path .
```

### Verificar instalación

```bash
lumen --version
# lumen 0.1.0

lumen --help
```

### Variables de entorno

| Variable | Descripción |
|----------|-------------|
| `LUMEN_ANY_OS` | Cualquier valor permite ejecutar en sistemas que no sean Linux |
| `RUST_BACKTRACE=1` | Muestra backtrace en caso de panic |

---

## Uso rápido

### Archivo simple
```bash
# Crear o abrir un archivo de texto
lumen mi_cuento.txt

# Lumen crea el archivo si no existe y lo abre para edición
```

### Dentro de Lumen
1. Escribes directamente — el cursor está en el texto
2. **Ctrl+G** → ir a línea específica
3. **Ctrl+F** → buscar texto
4. **Ctrl+S** → guardar
5. **Ctrl+Q** → salir

### Proyecto literario
```bash
# Abrir cualquier archivo de texto
lumen capitulo1.txt

# Dentro de Lumen:
# Ctrl+Shift+N → crear proyecto (título + autor)
# F5 → abrir panel creativo
```

---

## Atajos de teclado

### Edición

| Atajo | Acción |
|-------|--------|
| Ctrl+S | Guardar |
| Ctrl+Shift+S | Guardar como... |
| Ctrl+O | Abrir archivo... |
| Ctrl+Z | Deshacer |
| Ctrl+Y | Rehacer |
| Ctrl+A | Seleccionar todo |
| Ctrl+C | Copiar |
| Ctrl+X | Cortar |
| Ctrl+V | Pegar |
| Tab | Insertar tabulación |

### Navegación

| Atajo | Acción |
|-------|--------|
| Flechas | Mover cursor |
| Shift+Flechas | Seleccionar texto |
| Ctrl+Flechas ↑/↓ | Mover por párrafo |
| Ctrl+Home | Inicio del documento |
| Ctrl+End | Final del documento |
| Ctrl+W | Mover cursor una palabra a la izquierda |
| Ctrl+B | Mover cursor una palabra a la derecha |
| Ctrl+G | Ir a línea... |

### Búsqueda y reemplazo

| Atajo | Acción |
|-------|--------|
| Ctrl+F | Buscar... (F6 también) |
| Ctrl+H | Reemplazar... |
| Ctrl+Shift+F | Enfocar panel actual |
| Ctrl+Alt+R | Reemplazar todas las ocurrencias |

### Paneles laterales

| Atajo | Acción |
|-------|--------|
| F2 | Notas |
| F3 | Ortografía |
| F4 | Ideas |
| F5 | Creativo (panel unificado) |
| F6 | Buscar |

### Panel Creativo (F5)

| Tecla | Acción |
|-------|--------|
| 1-6 | Seleccionar sección directamente |
| ↑ ↓ | Navegar menú o lista |
| Enter | Abrir sección / Editar elemento |
| N | Crear nuevo elemento |
| E | Editar nombre del elemento seleccionado |
| D | Borrar elemento seleccionado |
| T | Ciclar estado de capítulo (solo en sección Capítulos) |
| Esc / F5 | Volver al editor |

### Navegación en menú

| Atajo | Acción |
|-------|--------|
| F10 | Abrir/cerrar menú |
| Alt+A | Menú Archivo |
| Alt+E | Menú Edición |
| Alt+B | Menú Buscar |
| ← → | Navegar menús |
| ↑ ↓ | Navegar items del menú |
| Enter | Ejecutar acción |

### En paneles de notas / ideas

| Tecla | Acción |
|-------|--------|
| N | Crear nueva nota/idea |
| Enter | Editar nota/idea seleccionada |
| D / Supr | Borrar nota/idea |
| ↑ ↓ | Navegar |
| Esc | Cerrar panel |

### En panel de ortografía (F3)

| Tecla | Acción |
|-------|--------|
| ↑ ↓ | Navegar errores |
| Enter | Ver sugerencias del error |
| Enter (en sugerencias) | Reemplazar palabra |
| L | Cambiar idioma |
| Esc / F3 | Cerrar panel |

---

## Proyectos

### Crear un proyecto

**Dentro de Lumen:**
1. Presionar **Ctrl+Shift+N** o ir a **Menú > Archivo > Nuevo proyecto...**
2. Ingresar el título del proyecto → Enter
3. Ingresar el autor → Enter
4. Se crea la estructura de directorios en la ubicación del archivo actual

**Estructura creada:**
```
mi-proyecto/
├── .lumen/
│   ├── project.toml      ← metadatos (título, autor, idioma)
│   ├── chapters.toml     ← índice de capítulos
│   └── creative.toml     ← contexto creativo
├── capitulo-1.txt
├── capitulo-2.txt
└── ...
```

### Abrir un proyecto existente

Lumen detecta automáticamente si un archivo pertenece a un proyecto. Al abrir un archivo con:

```bash
lumen mi-proyecto/capitulo-1.txt
```

Lumen busca hacia arriba en la estructura de directorios hasta encontrar `.lumen/project.toml`. Si lo encuentra, carga el proyecto automáticamente.

### Estados de capítulo

Cada capítulo puede tener uno de cuatro estados que se ciclan con **T**:

| Estado | Marcador | Descripción |
|--------|----------|-------------|
| Borrador | B | Texto en proceso |
| En revisión | R | Primer borrador completo, en revisión |
| Revisado | • | Revisión terminada |
| Finalizado | ✓ | Capítulo terminado |

### Manejo de capítulos

Desde el panel Creativo (F5 → sección 1):
- **N**: crear nuevo capítulo
- **T**: ciclar estado del capítulo seleccionado
- Los IDs de capítulo son estables: sobreviven reorden y eliminación
- Las asociaciones personajes/lugares/eventos usan estos IDs

---

## Panel Creativo

El panel Creativo (F5) es un centro de memoria externa para el escritor. Contiene seis secciones:

### 1. Capítulos (1)
Lista todos los capítulos del proyecto con su estado actual. Permite crear nuevos capítulos y ciclar sus estados.

### 2. Personajes (2)
Gestión de personajes de la historia.
- **N**: crear personaje
- **E**: editar nombre
- **D**: borrar personaje
- Cada personaje tiene: nombre, descripción, notas, IDs de capítulos asociados

### 3. Lugares (3)
Gestión de escenarios y ubicaciones.
- Mismo esquema CRUD que personajes
- Asociaciones con capítulos

### 4. Línea de tiempo (4)
Eventos cronológicos de la historia.
- Ordenados por campo numérico (puede ser año, epoch, o cualquier valor)
- Asociados con capítulos
- Ideales para mantener coherencia temporal

### 5. Conceptos (5)
Ideas temáticas, motivos, simbolismo.
- Nombre, descripción, notas
- Sin asociación directa con capítulos (conceptos son globales)

### 6. Estadísticas (6)
Vista de solo lectura con métricas del proyecto:
- Total de palabras
- Número de capítulos
- Promedio de palabras por capítulo
- Número de personajes, lugares, conceptos, eventos
- Porcentaje de capítulos finalizados

### Persistencia

Todos los datos creativos se guardan en `.lumen/creative.toml`:
- Formato TOML abierto y legible
- Escritura atómica (temp + rename)
- Carga perezosa (solo al abrir F5)
- Guardado al cerrar el panel

---

## Ortografía

### Uso
1. Presionar **F3** para abrir el panel de ortografía
2. Lumen escanea el documento actual y muestra errores
3. Navegar con ↑↓ y Enter para ver sugerencias
4. Enter sobre una sugerencia reemplaza la palabra
5. **L** para cambiar idioma
6. **A** para volver a detección automática

### Idiomas
Lumen busca diccionarios Hunspell (`.aff` / `.dic`) en:
- `/usr/share/hunspell/`
- `/usr/share/myspell/`
- `~/.local/share/lumen/dicts/` (usuario)

El idioma se configura en `~/.config/lumen/config.toml` con `language = "auto"` para detección automática, o un código ISO como `language = "es"`.

### Diccionario personal
Las palabras agregadas con "agregar al diccionario" se guardan en `~/.config/lumen/personal_dict.txt`, una palabra por línea.

---

## Formos de archivo

### TXT (nativo)
- Formo por defecto para nuevos archivos
- Guardado atómico (escritura a archivo temporal + rename)
- Soporte completo de UTF-8 con acentos y caracteres especiales

### ODT (OpenDocument)
- Lectura y escritura de archivos `.odt` (LibreOffice, OpenOffice)
- Implementado con `zip` y `quick-xml`
- Permite exportar desde Lumen para usar en otros procesadores
- Permite importar para continuar escribiendo en Lumen

### Detección automática
Lumen detecta el formato por extensión:
- `.odt` → formato ODT
- Todo lo demás → formato TXT

---

## Configuración

El archivo de configuración se encuentra en:

```
~/.config/lumen/config.toml
```

### Opciones

```toml
# Ancho del tabulador (por defecto: 4)
tab_width = 4

# Idioma para ortografía
# "auto" detecta automáticamente del sistema
# "es", "en", "fr", etc. fija un idioma específico
language = "auto"

# Habilitar/deshabilitar corrector ortográfico
spellcheck_enabled = true
```

Si el archivo no existe o tiene errores, Lumen usa valores por defecto. No es necesario crearlo manualmente — Lumen lo crea al primer guardado.

---

## Estructura del código

```
src/
├── main.rs              ← punto de entrada, argumentos CLI, loop principal
├── app/
│   ├── mod.rs           ← App: estado principal, manejo de eventos
│   └── browser.rs       ← navegador de archivos (diálogo Ctrl+O)
├── editor/
│   ├── mod.rs           ← Editor: cursor, selección, inserción, borrado
│   └── undo.rs          ← árbol de deshacer/rehacer
├── document/
│   ├── mod.rs           ← Document: apertura/guardado unificado
│   ├── model.rs         ← trait DocumentModel, formatos TXT/ODT
│   └── odt.rs           ← implementación ODT (zip + XML)
├── panels.rs            ← estado de los paneles (notas, ideas, ortografía, creativo)
├── project.rs           ← Project: creación, apertura, capítulos, persistencia
├── creative/
│   ├── mod.rs           ← CreativeContext: save/load, find/remove
│   ├── character.rs     ← Character struct
│   ├── place.rs         ← Place struct
│   ├── concept.rs       ← Concept struct
│   └── timeline.rs      ← TimelineEvent struct
├── spellcheck/
│   ├── mod.rs           ← detección de idiomas, carga de diccionarios
│   ├── engine.rs        ← SpellcheckEngine: escaneo, sugerencias, reemplazo
│   └── personal.rs      ← diccionario personal
├── search/
│   └── mod.rs           ← búsqueda y reemplazo
├── command.rs           ← Command enum: mapeo de teclas a comandos
├── config/
│   └── mod.rs           ← Config: carga/guardado de config.toml
├── backup.rs            ← autoguardado y recuperación
├── session.rs           ← persistencia de sesión (palabras escritas)
└── ui/
    └── mod.rs           ← renderizado con ratatui (todas las vistas)
```

### Dependencias

| Crate | Versión | Uso |
|-------|---------|-----|
| `crossterm` | 0.28 | Manejo de terminal, eventos de teclado |
| `ratatui` | 0.29 | Framework de UI para terminal (TUI) |
| `ropey` | 1.6 | Rope para edición eficiente de texto grande |
| `spellbook` | 0.4 | Corrector ortográfico (Hunspell-compatible) |
| `unicode-width` | 0.2 | Ancho correcto de caracteres Unicode |
| `zip` | 0.6 | Lectura/escritura de archivos ODT (ZIP) |
| `quick-xml` | 0.31 | Parsing de XML dentro de ODT |
| `toml` | 0.8 | Serialización/deserialización TOML |
| `serde` | 1 | Framework de serialización |

---

## Licencia

MIT License

```
Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
