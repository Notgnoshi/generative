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

    /// Parameters to define when the dynamical system is evaluated
    ///
    /// May be given multiple times. Each parameter should be defined as '-p name=value'
    ///
    /// Anything defined with --math or --script may override these parameters.
    #[clap(short, long)]
    parameter: Vec<String>,

    /// Letters A..=Y used to generate the values of parameters a_1, a_2, ..., a_n
    ///
    /// Example: "ABCD" will generate parameters a_1=-1.2, a_2=-1.1, a_3=-1.0, a_4=-0.9
    ///
    /// Example: "WXY" will generate parameters a_1=1.0, a_2=1.1, a_3=1.2
    ///
    /// Anything defined with --math or --script may override these parameters.
    #[clap(long)]
    letters: Option<String>,
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

fn params_from_letters(letters: &str) -> eyre::Result<Vec<String>> {
    let mut params = Vec::new();
    for (param_index, ch) in letters.chars().enumerate() {
        if !('A'..='Y').contains(&ch) {
            eyre::bail!("--letters may only contain letters A..=Y; found '{ch}'");
        }
        let alphabet_index = ((ch as u8) - b'A') as f64;
        let value = -1.2 + (alphabet_index * 0.1);

        let param_index = param_index + 1; // make 1-based
        params.push(format!("let a_{param_index} = {value};"));
    }

    Ok(params)
}

fn build_rune_script(args: &CmdlineOptions) -> eyre::Result<rune::Source> {
    let mut lines = Vec::new();
    lines.push("pub fn iterate(x, y) {".into());
    lines.push("// parameters".into());
    // Add the alphabetic parameters first
    if let Some(letters) = &args.letters {
        let mut params = params_from_letters(letters)?;
        lines.append(&mut params);
    }

    // Add the explicit parameters second (allows overrides if you so wanted)
    for parameter in &args.parameter {
        lines.push(format!("let {parameter};"));
    }

    // Add the script if given
    if let Some(script) = &args.script {
        lines.push("// script".into());
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

    // Finally append any --math expressions
    if !args.math.is_empty() {
        lines.push("// math".into());
    }
    lines.append(&mut args.math.clone());

    lines.push("return (x_new, y_new);".into());
    lines.push("}".into());
    let script = lines.join("\n");
    tracing::debug!("Generated rune script:\n==========\n{script}\n==========");
    let source = rune::Source::memory(script)?;
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
