// Spec 0005 stub: vibe-board-commit-check.
//
// Spec 0002 depends on this binary existing so the installed pre-commit
// hook can exec it. The full policy logic (TDD enforcement, file-count
// caps, agent-run identification) is owned by spec 0005. This stub
// satisfies the spec 0002 contract: invoking the hook with no staged
// changes exits 0. When spec 0005 is implemented, this file is rewritten
// in place — its location and binary name are part of spec 0005's
// Traceability, not spec 0002's.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            if o.stdout.iter().all(|b| b.is_ascii_whitespace()) {
                ExitCode::SUCCESS
            } else {
                // Spec 0005 will replace this with the real policy check.
                // For now the stub passes any non-empty staged tree too —
                // spec 0002's only assertion is the empty-tree case.
                ExitCode::SUCCESS
            }
        }
        _ => ExitCode::SUCCESS,
    }
}
