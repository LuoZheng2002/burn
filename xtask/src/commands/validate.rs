use tracel_xtask::prelude::*;

use crate::commands::{
    build::BurnBuildCmdArgs,
    test::{BurnTestCmdArgs, CiTestType},
};

pub fn handle_command(
    args: &ValidateCmdArgs,
    env: Environment,
    context: Context,
) -> anyhow::Result<()> {
    let target = Target::Workspace;
    let exclude = vec![];
    let only = vec![];

    if context == Context::NoStd || context == Context::All {
        // =================
        // no-std validation
        // =================
        info!("Run validation for no-std execution environment...");

        #[cfg(target_os = "linux")]
        {
            // build
            super::build::handle_command(
                BurnBuildCmdArgs {
                    target: target.clone(),
                    exclude: exclude.clone(),
                    only: only.clone(),
                    ci: true,
                    release: args.release,
                    features: args.features.clone(),
                    no_default_features: args.no_default_features,
                },
                env.clone(),
                Context::NoStd,
            )?;

            // tests
            super::test::handle_command(
                BurnTestCmdArgs {
                    target: target.clone(),
                    exclude: exclude.clone(),
                    only: only.clone(),
                    threads: None,
                    jobs: None,
                    command: Some(TestSubCommand::All),
                    ci: CiTestType::GithubRunner,
                    features: None,
                    no_default_features: false,
                    force: false,
                    no_capture: false,
                    release: args.release,
                    test: None,
                    miri: false,
                },
                env.clone(),
                Context::NoStd,
            )?;
        }
    }

    if context == Context::Std || context == Context::All {
        // ==============
        // std validation
        // ==============
        info!("Run validation for std execution environment...");

        // checks
        [
            CheckSubCommand::Audit,
            CheckSubCommand::Format,
            CheckSubCommand::Lint,
            CheckSubCommand::Typos,
        ]
        .iter()
        .try_for_each(|c| {
            let mut check_args = CheckCmdArgs {
                target: target.clone(),
                exclude: exclude.clone(),
                only: only.clone(),
                command: Some(c.clone()),
                ignore_audit: args.ignore_audit,
                features: args.features.clone(),
                no_default_features: args.no_default_features,
                ignore_typos: args.ignore_typos,
            };

            if *c == CheckSubCommand::Lint {
                // Keep validate's lint path aligned with the temporary check patch
                // in main.rs until Check is fully moved into commands::check.
                let should_lint_burn_tch =
                    !check_args.exclude.iter().any(|krate| krate == "burn-tch")
                        && (check_args.only.is_empty()
                            || check_args.only.iter().any(|krate| krate == "burn-tch"))
                        && (check_args.target == Target::Workspace
                            || check_args.target == Target::Crates);

                if check_args.target == Target::Workspace {
                    check_args.target = Target::Crates;
                }

                if !check_args.exclude.iter().any(|krate| krate == "burn-tch") {
                    check_args.exclude.push("burn-tch".to_string());
                }

                if should_lint_burn_tch {
                    let lint_args = vec![
                        "clippy".to_string(),
                        "--no-deps".to_string(),
                        "--color=always".to_string(),
                        "-p".to_string(),
                        "burn-tch".to_string(),
                        "--features".to_string(),
                        "download-libtorch".to_string(),
                        "--".to_string(),
                        "--deny".to_string(),
                        "warnings".to_string(),
                    ];

                    let lint_args_ref: Vec<&str> = lint_args.iter().map(String::as_str).collect();

                    run_process(
                        "cargo",
                        &lint_args_ref,
                        None,
                        None,
                        "burn-tch lint should pass with download-libtorch",
                    )?;
                }
            }

            base_commands::check::handle_command(check_args, env.clone(), context.clone())
        })?;

        // build
        super::build::handle_command(
            BurnBuildCmdArgs {
                target: target.clone(),
                exclude: exclude.clone(),
                only: only.clone(),
                ci: true,
                release: args.release,
                features: args.features.clone(),
                no_default_features: args.no_default_features,
            },
            env.clone(),
            Context::Std,
        )?;

        // tests
        super::test::handle_command(
            BurnTestCmdArgs {
                target: target.clone(),
                exclude: exclude.clone(),
                only: only.clone(),
                threads: None,
                jobs: None,
                command: Some(TestSubCommand::All),
                ci: CiTestType::GithubRunner,
                features: None,
                no_default_features: false,
                release: args.release,
                test: None,
                force: false,
                no_capture: false,
                miri: false,
            },
            env.clone(),
            Context::Std,
        )?;

        // documentation
        [DocSubCommand::Build, DocSubCommand::Tests]
            .iter()
            .try_for_each(|c| {
                super::doc::handle_command(
                    DocCmdArgs {
                        target: target.clone(),
                        exclude: exclude.clone(),
                        only: only.clone(),
                        command: Some(c.clone()),
                        features: args.features.clone(),
                        no_default_features: args.no_default_features,
                    },
                    env.clone(),
                    context.clone(),
                )
            })?;
    }

    Ok(())
}
