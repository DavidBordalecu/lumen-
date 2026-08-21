mod app;
mod backup;
mod command;
mod config;
mod creative;
mod document;
mod editor;
mod panels;
mod project;
mod search;
mod session;
mod spellcheck;
mod ui;

use std::io;
use std::time::Duration;

use app::App;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let file = parse_args(&args)?;

    if cfg!(not(target_os = "linux")) && std::env::var("LUMEN_ANY_OS").is_err() {
        eprintln!("Lumen está diseñado exclusivamente para Linux. Este sistema no es compatible.");
        eprintln!("Para probarlo en desarrollo, define la variable de entorno LUMEN_ANY_OS=1.");
        std::process::exit(1);
    }

    let _guard = TermGuard;

    let mut terminal = ratatui::init();
    // Las flags de teclado (protocolo kitty) y el bracketed paste son mejor
    // esfuerzo: si la terminal no los soporta, Lumen sigue funcionando sin
    // ellas (por ejemplo, en consolas Windows legacy).
    let _ = execute!(
        io::stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        ),
        EnableBracketedPaste
    );

    let mut app = App::new(file);
    terminal.draw(|f| ui::draw(f, &mut app))?;

    loop {
        if app.exit {
            break;
        }
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key);
                    terminal.draw(|f| ui::draw(f, &mut app))?;
                }
                Event::Paste(text) => {
                    app.handle_paste(text);
                    terminal.draw(|f| ui::draw(f, &mut app))?;
                }
                Event::Resize(_, _) => {
                    terminal.draw(|f| ui::draw(f, &mut app))?;
                }
                _ => {}
            }
        }
        app.tick();
    }

    Ok(())
}

/// Restaura la terminal aunque el programa termine por un error o un pánico.
struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            PopKeyboardEnhancementFlags,
            DisableBracketedPaste
        );
        ratatui::restore();
    }
}

fn parse_args(args: &[String]) -> io::Result<Option<String>> {
    let mut file = None;
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                print!(
                    "Lumen — procesador de textos para terminal Linux\n\n\
                     USO:\n  lumen [archivo.txt]\n\n\
                     ATAJOS:\n  Ctrl+S         Guardar\n  Ctrl+Shift+S   Guardar como\n  \
                     Ctrl+O         Abrir\n  Ctrl+Shift+N   Nuevo proyecto\n  \
                     Ctrl+Z         Deshacer\n  Ctrl+Y         Rehacer\n  \
                     Ctrl+F         Buscar\n  Ctrl+H         Reemplazar\n  Ctrl+G       Ir a línea\n  \
                     Ctrl+A         Seleccionar todo\n  Ctrl+Shift+F   Focus\n  Ctrl+Q       Salir\n  \
                     F2             Notas\n  F3             Ortografía\n  F4             Ideas\n  \
                     F5             Creativo\n  F6             Buscar\n"
                );
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("lumen {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            s if s.starts_with('-') => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("opción desconocida: {s}"),
                ));
            }
            s => {
                if file.is_none() {
                    file = Some(s.to_string());
                }
            }
        }
    }
    Ok(file)
}
