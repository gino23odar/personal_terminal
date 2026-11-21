use crate::{
    CancellationToken, CommandContext, CommandIo, CommandRegistry, CommandResult, OutputStream,
    ShellError, ShellSession, parse,
};

/// Execute one logical line. The adapter owns polling, I/O and a fresh token per call.
/// Exclusive session borrowing serializes executions without platform locks.
pub async fn execute_line(
    input: &str,
    registry: &CommandRegistry,
    session: &mut ShellSession,
    io: &mut dyn CommandIo,
    cancellation: &CancellationToken,
) -> CommandResult {
    if let Some(status) = session.exit_status() {
        return Ok(status);
    }
    let result = cancellation
        .run_until_cancelled(async {
            let result = dispatch(input, registry, session, io, cancellation).await;
            if let Err(error) = &result {
                // Failed transports are returned to the adapter, never retried recursively.
                if !matches!(error, ShellError::Cancelled | ShellError::Io(_)) {
                    let message = format!("gino: {}\n", error.to_string().escape_debug());
                    io.write(OutputStream::Stderr, message.as_bytes()).await?;
                }
            }
            result
        })
        .await
        .and_then(|result| result);
    session.record_status(match &result {
        Ok(status) => *status,
        Err(error) => error.exit_status(),
    });
    result
}

async fn dispatch(
    input: &str,
    registry: &CommandRegistry,
    session: &mut ShellSession,
    io: &mut dyn CommandIo,
    cancellation: &CancellationToken,
) -> CommandResult {
    let Some(command) = parse(input).map_err(ShellError::Syntax)? else {
        return Ok(session.last_status());
    };
    let name = command.name.value();
    let entry = registry
        .get(&name)
        .ok_or(ShellError::UnknownCommand(name))?;
    let args: Vec<_> = command.arguments.iter().map(|word| word.value()).collect();
    entry
        .handler()
        .execute(
            &args,
            CommandContext {
                session,
                cancellation,
                registry,
            },
            io,
        )
        .await
}
