use alloc::{borrow::Cow, boxed::Box, format, string::String, vec::Vec};
use core::{error::Error, fmt};

#[derive(Clone, Debug)]
pub struct ShaderError<E> {
    /// The source code of the shader.
    pub source: String,
    pub label: Option<String>,
    pub inner: Box<E>,
}

#[cfg(feature = "wgsl-in")]
impl fmt::Display for ShaderError<crate::front::wgsl::ParseError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.label.as_deref().unwrap_or_default();
        let string = self.inner.emit_to_string(&self.source);
        write!(f, "\nShader '{label}' parsing {string}")
    }
}

#[cfg(feature = "glsl-in")]
impl fmt::Display for ShaderError<crate::front::glsl::ParseErrors> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.label.as_deref().unwrap_or_default();
        let string = self.inner.emit_to_string(&self.source);
        write!(f, "\nShader '{label}' parsing {string}")
    }
}

#[cfg(feature = "spv-in")]
impl fmt::Display for ShaderError<crate::front::spv::Error> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.label.as_deref().unwrap_or_default();
        let string = self.inner.emit_to_string(&self.source);
        write!(f, "\nShader '{label}' parsing {string}")
    }
}

impl fmt::Display for ShaderError<crate::WithSpan<crate::valid::ValidationError>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use codespan_reporting::{files::SimpleFile, term};

        let label = self.label.as_deref().unwrap_or_default();
        let files = SimpleFile::new(label, replace_control_chars(&self.source));
        let config = term::Config::default();

        let writer = {
            let mut writer = DiagnosticBuffer::new();
            term::emit(
                writer.inner_mut(),
                &config,
                &files,
                &self.inner.diagnostic(),
            )
            .expect("cannot write error");
            writer.into_string()
        };

        write!(f, "\nShader validation {writer}")
    }
}

type DiagnosticBufferInner = codespan_reporting::term::termcolor::NoColor<Vec<u8>>;
pub(crate) use codespan_reporting::term::termcolor::WriteColor as _ErrorWrite;

#[cfg_attr(
    not(any(feature = "spv-in", feature = "glsl-in")),
    expect(
        unused_imports,
        reason = "only need `ErrorWrite` with an appropriate front-end."
    )
)]
pub(crate) use _ErrorWrite as ErrorWrite;

pub(crate) struct DiagnosticBuffer {
    inner: DiagnosticBufferInner,
}

impl DiagnosticBuffer {
    pub fn new() -> Self {
        Self {
            inner: codespan_reporting::term::termcolor::NoColor::new(Vec::new()),
        }
    }

    pub fn inner_mut(&mut self) -> &mut DiagnosticBufferInner {
        &mut self.inner
    }

    pub fn into_string(self) -> String {
        String::from_utf8(self.inner.into_inner()).unwrap()
    }
}

impl<E> ShaderError<E>
where
    E: fmt::Display,
{
    pub fn emit_to_stderr(&self, kind: &str) {
        #[cfg(feature = "stderr")]
        {
            eprintln!(
                "{}",
                self.format_to_string(kind, &codespan_reporting::term::Config::default())
            );
        }
        #[cfg(not(feature = "stderr"))]
        {
            let _ = kind;
        }
    }

    pub fn format_to_string(
        &self,
        kind: &str,
        _config: &codespan_reporting::term::Config,
    ) -> String {
        let label = self.label.as_deref().unwrap_or_default();
        format!("\nShader '{label}' {kind} {}", self.inner)
    }
}

impl<E> Error for ShaderError<E>
where
    E: Error + fmt::Display + 'static,
    ShaderError<E>: fmt::Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

pub(crate) fn replace_control_chars(input: &str) -> Cow<'_, str> {
    if input.contains(char::is_control) {
        Cow::Owned(
            input
                .chars()
                .map(|ch| if ch.is_control() { ' ' } else { ch })
                .collect(),
        )
    } else {
        Cow::Borrowed(input)
    }
}
