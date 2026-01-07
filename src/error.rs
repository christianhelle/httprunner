/// Type alias for Result with boxed error
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Extension trait to add context to errors (similar to anyhow::Context)
pub trait Context<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(ContextError {
                context: context.to_string(),
                source: Box::new(e),
            })
        })
    }
}

#[derive(Debug)]
struct ContextError {
    context: String,
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Macro to create simple errors from format strings (similar to anyhow::anyhow!)
#[macro_export]
macro_rules! err {
    ($msg:literal $(,)?) => {
        Box::<dyn std::error::Error + Send + Sync>::from($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        Box::<dyn std::error::Error + Send + Sync>::from(format!($fmt, $($arg)*))
    };
}
