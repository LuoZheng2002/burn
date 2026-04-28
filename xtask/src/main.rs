mod commands;

#[macro_use]
extern crate log;

use std::time::Instant;
use tracel_xtask::prelude::*;

// no-std
const WASM32_TARGET: &str = "wasm32-unknown-unknown";
const ARM_TARGET: &str = "thumbv7m-none-eabi";
const ARM_NO_ATOMIC_PTR_TARGET: &str = "thumbv6m-none-eabi";
const NO_STD_CRATES: &[&str] = &[
    "burn",
    "burn-autodiff",
    "burn-core",
    "burn-std",
    "burn-backend",
    "burn-tensor",
    "burn-flex",
    "burn-ndarray",
    "burn-no-std-tests",
];

#[macros::base_commands(
    Bump,
    Check,
    Compile,
    Coverage,
    Doc,
    Dependencies,
    Fix,
    Publish,
    Validate,
    Vulnerabilities
)]
pub enum Command {
    /// Run commands to manage Burn Books.
    Books(commands::books::BooksArgs),
    /// Build Burn in different modes.
    Build(commands::build::BurnBuildCmdArgs),
    /// Test Burn.
    Test(commands::test::BurnTestCmdArgs),
}

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let (args, environment) = init_xtask::<Command>(parse_args::<Command>()?)?;

    if args.context == Context::NoStd {
        // Install additional targets for no-std execution environments
        rustup_add_target(WASM32_TARGET)?;
        rustup_add_target(ARM_TARGET)?;
        rustup_add_target(ARM_NO_ATOMIC_PTR_TARGET)?;
    }

    match args.command {
        Command::Books(cmd_args) => cmd_args.parse(),
        Command::Build(cmd_args) => {
            commands::build::handle_command(cmd_args, environment, args.context)
        }
        Command::Check(mut cmd_args) => {
            // Temporary compatibility patch for burn-tch after making
            // `download-libtorch` explicit.
            //
            // A cleaner long-term design is to remove `Check` from
            // `macros::base_commands` and implement `commands::check::handle_command`
            // (like Build/Test/Doc). That is a larger behavior-moving refactor in
            // xtask dispatch, so we keep this narrow patch for now and defer the
            // full restructuring to core contributors.
            if matches!(cmd_args.command, Some(CheckSubCommand::Lint)) {
                let has_excluded_burn_tch = cmd_args.exclude.iter().any(|krate| krate == "burn-tch");
                let has_only_burn_tch = cmd_args.only.iter().any(|krate| krate == "burn-tch");

                let has_excluded_multinode_tests =
                    cmd_args.exclude.iter().any(|krate| krate == "multinode-tests");
                let has_excluded_burn_collective_multinode_tests = cmd_args
                    .exclude
                    .iter()
                    .any(|krate| krate == "burn-collective-multinode-tests");
                let has_only_multinode_tests =
                    cmd_args.only.iter().any(|krate| krate == "multinode-tests");
                let has_only_burn_collective_multinode_tests = cmd_args
                    .only
                    .iter()
                    .any(|krate| krate == "burn-collective-multinode-tests");

                let should_lint_burn_tch =
                    !has_excluded_burn_tch
                        && (cmd_args.only.is_empty() || has_only_burn_tch)
                        && (cmd_args.target == Target::Workspace
                            || cmd_args.target == Target::Crates);

                // Same workaround pattern for `multinode-tests`: base lint resolves
                // workspace members by path stem, but cargo expects package name.
                let should_lint_multinode_tests =
                    !has_excluded_multinode_tests
                        && !has_excluded_burn_collective_multinode_tests
                        && (cmd_args.only.is_empty()
                            || has_only_multinode_tests
                            || has_only_burn_collective_multinode_tests)
                        && (cmd_args.target == Target::Workspace
                            || cmd_args.target == Target::Crates);

                if cmd_args.target == Target::Workspace {
                    cmd_args.target = Target::Crates;
                }

                if !has_excluded_burn_tch {
                    cmd_args.exclude.push("burn-tch".to_string());
                }

                if !has_excluded_multinode_tests {
                    cmd_args.exclude.push("multinode-tests".to_string());
                }

                if should_lint_burn_tch {
                    let lint_args = [
                        "clippy",
                        "--no-deps",
                        "--color=always",
                        "-p",
                        "burn-tch",
                        "--features",
                        "download-libtorch",
                        "--",
                        "--deny",
                        "warnings",
                    ];

                    run_process(
                        "cargo",
                        &lint_args,
                        None,
                        None,
                        "burn-tch lint should pass with download-libtorch",
                    )?;
                }

                if should_lint_multinode_tests {
                    let lint_args = [
                        "clippy",
                        "--no-deps",
                        "--color=always",
                        "-p",
                        "burn-collective-multinode-tests",
                        "--",
                        "--deny",
                        "warnings",
                    ];

                    run_process(
                        "cargo",
                        &lint_args,
                        None,
                        None,
                        "multinode-tests lint should pass with package name",
                    )?;
                }

                base_commands::check::handle_command(cmd_args, environment.clone(), args.context)?;

                Ok(())
            } else {
                base_commands::check::handle_command(cmd_args, environment, args.context)
            }
        }
        Command::Doc(cmd_args) => {
            commands::doc::handle_command(cmd_args, environment, args.context)
        }
        Command::Test(cmd_args) => {
            commands::test::handle_command(cmd_args, environment, args.context)
        }
        Command::Validate(cmd_args) => {
            commands::validate::handle_command(&cmd_args, environment, args.context)
        }
        _ => dispatch_base_commands(args, environment),
    }?;

    let duration = start.elapsed();
    info!(
        "\x1B[32;1mTime elapsed for the current execution: {}\x1B[0m",
        format_duration(&duration)
    );

    Ok(())
}
