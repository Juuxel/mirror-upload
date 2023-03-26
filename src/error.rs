/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::Display;
use miette::{Diagnostic, LabeledSpan, Report, SourceCode, SourceSpan};
use thiserror::Error;

/// An error that is created using the builder pattern.
///
/// New errors are created using [`MuError::new`],
/// while another layer of context can be added using [`MuError::and_then`].
///
/// An error can be converted into a [`Report`] using [`MuError::to_report`].
#[derive(Error, Debug)]
#[error("{msg}")]
pub struct MuError {
    msg: String,
    source_code: Option<String>,
    span: Option<SourceSpan>,
    help: Option<String>,
    // Note: the Send + Sync bound is needed for MuError to be Send + Sync,
    // which is needed for miette's From<MuError> for Report.
    #[source]
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl MuError {
    /// Creates a new error with a specified message.
    pub fn new<M: AsRef<str>>(msg: M) -> Self {
        MuError {
            msg: msg.as_ref().to_string(),
            source_code: None,
            span: None,
            help: None,
            cause: None,
        }
    }

    /// Wraps this error with another message for context.
    /// The resulting error copies this error's source code and
    /// source span, and is caused by this error.
    pub fn and_then<M: AsRef<str>>(self, msg: M) -> Self {
        Self::new(msg)
            .source_code(self.source_code.clone())
            .span(self.span.clone())
            .cause(self)
    }

    /// Sets the source code of this error.
    pub fn source_code<O: ToOption<String>>(mut self, source_code: O) -> Self
    {
        self.source_code = source_code.to_option();
        self
    }

    /// Sets the source span of this error.
    pub fn span<O: ToOption<SourceSpan>>(mut self, span: O) -> Self {
        self.span = span.to_option();
        self
    }

    /// Sets the help message of this error.
    pub fn help<O: ToOption<String>>(mut self, help: O) -> Self
    {
        self.help = help.to_option();
        self
    }

    /// Sets the cause of this error.
    pub fn cause<E>(mut self, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static
    {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Converts this error into a [`Report`].
    pub fn to_report(self) -> Report {
        Report::from(self)
    }
}

// Cannot derive because the derive macro doesn't support
// optional source codes.
impl Diagnostic for MuError {
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(|help| Box::new(help) as Box<dyn Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        if let Some(source_code) = &self.source_code {
            Some(source_code as &dyn SourceCode)
        } else {
            None
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item=LabeledSpan> + '_>> {
        self.span.map(|span| {
            let labeled = LabeledSpan::new_with_span(Some("here".to_string()), span);
            Box::new(vec![labeled].into_iter()) as Box<dyn Iterator<Item = LabeledSpan> + '_>
        })
    }
}

/// Something that can be converted into an [`Option<T>`].
/// Used for many inputs to [`MuError`] builder functions.
pub trait ToOption<T> {
    /// Converts this value into an [`Option<T>`].
    fn to_option(self) -> Option<T>;
}

impl<T> ToOption<T> for T {
    fn to_option(self) -> Option<T> {
        Some(self)
    }
}

impl<T> ToOption<T> for Option<T> {
    fn to_option(self) -> Option<T> {
        self
    }
}

impl<S: AsRef<str> + ?Sized> ToOption<String> for &S {
    fn to_option(self) -> Option<String> {
        Some(self.as_ref().to_string())
    }
}

impl<S: AsRef<str> + ?Sized> ToOption<String> for Option<&S> {
    fn to_option(self) -> Option<String> {
        self.map(|s| s.as_ref().to_string())
    }
}
