fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("__run_workflow_machine_real") {
        args.remove(0);
        match codex_governance_workbench_lib::run_workflow_machine_cli(args) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }

    if args.first().map(String::as_str) == Some("__mcp_server") {
        args.remove(0);
        if let Err(error) = codex_governance_workbench_lib::run_mcp_server_cli(args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    if args.first().map(String::as_str) == Some("__syn_bridge") {
        args.remove(0);
        if let Err(error) = codex_governance_workbench_lib::run_syn_bridge_cli(args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    codex_governance_workbench_lib::run()
}
