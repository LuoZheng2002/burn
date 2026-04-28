use tracel_xtask::prelude::*;

pub(crate) fn handle_command(
    mut args: DocCmdArgs,
    env: Environment,
    ctx: Context,
) -> anyhow::Result<()> {
    if args.get_command() == DocSubCommand::Build {
        // burn-tch docs are built in a dedicated pass below with
        // `download-libtorch` enabled.
        args.exclude.extend(vec![
            "burn-cuda".to_string(),
            "burn-rocm".to_string(),
            "burn-tch".to_string(),
        ]);

        // Workspace docs still resolve `burn-tch` through `burn` defaults.
        // Keep libtorch download explicit by forwarding the feature from burn.
        args.features.push("burn/download-libtorch".to_string());
    }

    if args.get_command() == DocSubCommand::Tests {
        // burn-tch doc tests are run in a dedicated pass below with
        // `download-libtorch` enabled.
        args.exclude.extend(vec!["burn-tch".to_string()]);

        // For the same reason as doc build, ensure workspace doc tests can
        // resolve the libtorch dependency path without implicit behavior.
        args.features.push("burn/download-libtorch".to_string());
    }

    // Execute documentation command on workspace
    base_commands::doc::handle_command(args.clone(), env, ctx)?;

    // Specific additional commands to build other docs
    if args.get_command() == DocSubCommand::Build {
        // burn-dataset
        helpers::custom_crates_doc_build(
            vec!["burn-dataset"],
            vec!["--all-features"],
            None,
            None,
            "All features",
        )?;

        // burn-tch
        helpers::custom_crates_doc_build(
            vec!["burn-tch"],
            vec!["--features", "download-libtorch"],
            None,
            None,
            "download-libtorch feature",
        )?;
    }

    if args.get_command() == DocSubCommand::Tests {
        helpers::custom_crates_tests(
            vec!["burn-tch"],
            vec!["--doc", "--features", "download-libtorch"],
            None,
            None,
            "doc tests with download-libtorch feature",
        )?;
    }
    Ok(())
}
