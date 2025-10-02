use std::io::BufRead;

use clap::Parser;

/// Attractor; runs dynamical systems
///
/// If neither --math nor --script are provided, a default dynamical system is provided for
/// prototyping.
#[derive(Debug, Parser)]
#[clap(name = "attractor", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Mathematical expressions defining the dynamical system
    ///
    /// The --math argument may be provided multiple times. The initial values of x and y will be
    /// populated, and the expression is expected to define new variables x_new and y_new.
    #[clap(short, long)]
    math: Vec<String>,

    /// The path to a rune script defining the dynamical system
    ///
    /// Has the same rules as --math. May be combined with --math (all --math arguments are
    /// appended to the end of the script).
    #[clap(short, long)]
    script: Option<String>,
}

type DynamicalSystemFn = Box<dyn Fn(f64, f64) -> (f64, f64) + 'static>;

fn build_dynamical_system_function(args: &CmdlineOptions) -> eyre::Result<DynamicalSystemFn> {
    // Interpret the CLI arguments to build the dynamical system function
    if !args.math.is_empty() || args.script.is_some() {
        build_dynamical_system_function_from_args(args)
    } else {
        // If no --math arguments are provided, use a default function useful for prototyping
        Ok(Box::new(|x, y| (x + 1.0, x * y - y)))
    }
}

fn build_dynamical_system_function_from_args(
    args: &CmdlineOptions,
) -> eyre::Result<DynamicalSystemFn> {
    let context = rune::Context::with_default_modules()?;
    let runtime = rune::sync::Arc::try_new(context.runtime()?)?;
    let mut script = rune::Sources::new();
    script.insert(build_rune_script(args)?)?;
    let mut diagnostics = rune::Diagnostics::new();
    let maybe_unit = rune::prepare(&mut script)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    if !diagnostics.is_empty() {
        let mut writer =
            rune::termcolor::StandardStream::stderr(rune::termcolor::ColorChoice::Always);
        diagnostics.emit(&mut writer, &script)?;
    }
    let unit = rune::sync::Arc::try_new(maybe_unit?)?;

    // the Vm is captured and retains state between calls
    let vm = rune::Vm::new(runtime, unit);
    let iterate = vm.lookup_function(["iterate"])?;
    let closure = move |x: f64, y: f64| -> (f64, f64) {
        iterate
            .call((x, y))
            .expect("Failed to call iterate function")
    };
    Ok(Box::new(closure))
}

fn build_rune_script(args: &CmdlineOptions) -> eyre::Result<rune::Source> {
    // TODO: There should be a way to define parameters from --letters
    // TODO: There should be a way to define parameters from --parameter

    let mut lines = vec!["pub fn iterate(x, y) {".to_string()];
    if let Some(script) = &args.script {
        if script == "-" {
            for maybe_line in std::io::stdin().lock().lines() {
                let line = maybe_line?;
                lines.push(line);
            }
        } else {
            let file = std::fs::File::open(script)?;
            let reader = std::io::BufReader::new(file);
            for maybe_line in reader.lines() {
                let line = maybe_line?;
                lines.push(line);
            }
        }
    }
    lines.append(&mut args.math.clone());
    lines.append(&mut vec!["return (x_new, y_new); }".to_string()]);
    let func = lines.join("\n");

    let source = rune::Source::memory(func)?;
    Ok(source)
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = CmdlineOptions::parse();

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(args.log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let dynamical_system = build_dynamical_system_function(&args)?;

    let mut x = 0.1;
    let mut y = -0.01;
    for _ in 0..10 {
        (x, y) = dynamical_system(x, y);
        println!("({}, {})", x, y);
    }

    Ok(())
}
