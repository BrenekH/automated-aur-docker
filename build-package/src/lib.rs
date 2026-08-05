pub fn build_package() {
    build_pkg().unwrap(); // TODO: Handle errors
}

fn build_pkg() -> anyhow::Result<()> {
    Ok(())
}

fn install_aur_deps(deps: &[&str]) -> anyhow::Result<()> {
    if deps.is_empty() {
        return Ok(());
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let mut paru_args = vec![
                "--mflags",
                "--skippgpcheck",
                "--nocheck",
                "--noconfirm",
                "-Syu",
            ];
            paru_args.extend(deps);

            paru::run(&paru_args).await;
        });

    Ok(())
}
