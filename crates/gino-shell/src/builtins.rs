use crate::{
    Command, CommandContext, CommandIo, CommandMetadata, CommandRegistry, CommandResult,
    ExitStatus, LocalFuture, OutputStream, RegistryError, ShellError, TerminalEvent,
};

/// A fresh registry containing exactly the implemented core commands.
pub fn builtin_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    for (name, summary, usage, handler) in [
        (
            "help",
            "List commands or describe one command",
            "help [command]",
            Box::new(Help) as Box<dyn Command>,
        ),
        (
            "echo",
            "Print arguments as literal text",
            "echo [--] [text ...]",
            Box::new(Echo),
        ),
        (
            "pwd",
            "Print the virtual working directory",
            "pwd",
            Box::new(Pwd),
        ),
        (
            "whoami",
            "Print the virtual user",
            "whoami",
            Box::new(Whoami),
        ),
        ("clear", "Request a clear screen", "clear", Box::new(Clear)),
        (
            "exit",
            "Close the virtual session",
            "exit [status]",
            Box::new(Exit),
        ),
    ] {
        registry
            .register(
                CommandMetadata {
                    name: name.into(),
                    summary: summary.into(),
                    usage: usage.into(),
                    aliases: vec![],
                },
                handler,
            )
            .expect("built-in metadata is valid and unique");
    }
    registry
}

fn usage(message: &str) -> ShellError {
    ShellError::InvalidInput(format!("usage: {message}"))
}
fn no_args(args: &[String], name: &str) -> Result<(), ShellError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(usage(name))
    }
}
async fn output(io: &mut dyn CommandIo, text: &str) -> CommandResult {
    io.write(OutputStream::Stdout, text.as_bytes()).await?;
    Ok(ExitStatus::SUCCESS)
}

struct Help;
impl Command for Help {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            if args.len() > 1 || args.first().is_some_and(|arg| arg.starts_with('-')) {
                return Err(usage("help [command]"));
            }
            let text = context
                .registry
                .help_text(args.first().map(String::as_str))
                .map_err(|error| match error {
                    RegistryError::UnknownCommand(name) => ShellError::UnknownCommand(name),
                    other => ShellError::InvalidInput(other.to_string()),
                })?;
            output(io, &text).await
        })
    }
}
struct Echo;
impl Command for Echo {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        _: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            let args = if args.first().is_some_and(|arg| arg == "--") {
                &args[1..]
            } else {
                if args
                    .first()
                    .is_some_and(|arg| arg.starts_with('-') && arg != "-")
                {
                    return Err(usage("echo [--] [text ...] (options are not supported)"));
                }
                args
            };
            output(io, &format!("{}\n", args.join(" "))).await
        })
    }
}
struct Pwd;
impl Command for Pwd {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            no_args(args, "pwd")?;
            output(io, &format!("{}\n", context.session.cwd())).await
        })
    }
}
struct Whoami;
impl Command for Whoami {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        _: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            no_args(args, "whoami")?;
            output(io, "visitor\n").await
        })
    }
}
struct Clear;
impl Command for Clear {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        _: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            no_args(args, "clear")?;
            io.emit(TerminalEvent::ClearScreen).await?;
            Ok(ExitStatus::SUCCESS)
        })
    }
}
struct Exit;
impl Command for Exit {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        _: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            let status = match args {
                [] => context.session.last_status(),
                [value] if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) => {
                    ExitStatus(
                        value
                            .parse::<u8>()
                            .map_err(|_| usage("exit [status: 0..255]"))?,
                    )
                }
                _ => return Err(usage("exit [status: 0..255]")),
            };
            context.session.close(status);
            Ok(status)
        })
    }
}
