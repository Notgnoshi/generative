use clap::Parser;

/// Attractor; runs dynamical systems
#[derive(Debug, Parser)]
#[clap(name = "attractor", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    #[clap(short, long)]
    math: Vec<String>,
}

type DynamicalSystemFn = Box<dyn Fn(f64, f64) -> (f64, f64) + 'static>;

fn build_dynamical_system_function(args: &CmdlineOptions) -> eyre::Result<DynamicalSystemFn> {
    // Interpret the CLI arguments to build the dynamical system function
    if !args.math.is_empty() {
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
    // TODO: There should be a way to define all of the a_i parameters

    let mut lines = vec!["pub fn iterate(x, y) {".to_string()];
    // TODO: I also want to be able to read from stdin or a .rn script. In those cases, should the
    // whole fn be provided, or just the body?
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
