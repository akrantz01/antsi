use pyo3::prelude::*;

#[cfg(test)]
#[macro_use]
mod macros;
mod ast;
mod color;
mod lexer;
mod parser;

/// Convert styled markup to ANSI escape codes.
///
/// Converts styled markup within the source text to ANSI escape codes allowing text to be formatted
/// on the command line. If a string has no styled markup, it will be passed through unchanged. Any
/// invalid/unparseable markup will cause an exception.
///
/// Styled markup is defined as follows:
/// ```text
/// [ <style specifiers> ]( <content> )
///
/// <style specifiers> ::= <style specifier>;+
///  <style specifier> ::= <tag> : <value>
///          <content> ::= any character except \, [, ], (, )
///              <tag> ::= (see below)
///            <value> ::= (see below)
/// ```
///
/// # Tags
///
/// There are three different ways that styling can be applied: foreground color, background color,
/// and text decoration. By default, text hsa no styling applied.
///
/// ## Foreground color (`fg`)
///
/// Accepted values: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
///
/// Controls the foreground color of the text. Colors can be made more intense using the `bright-`
/// prefix.
///
/// ## Background color (`bg`)
///
/// Accepted values: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
///
/// Controls the background color of the text. Colors can be made more intense using the `bright-`
/// prefix.
///
/// ## Text decoration (`deco`)
///
/// Accepted values: `bold`, `dim`, `italic`, `underline`, `fast-blink`, `slow-blink`, `invert`,
/// `hide`, `strike-through`
///
/// Controls additional text decoration. Multiple text decorations can be applied by separating the
/// styles with a comma (i.e. `deco:bold,italic`).
///
/// # Escape sequences
///
/// Certain control characters must be escaped to include them in your text. The valid escape
/// sequences are as follows:
///
/// |Sequence|Character|
/// |:-:|:-:|
/// |`\\`|`\`|
/// |`\[`|`[`|
/// |`\]`|`]`|
/// |`\(`|`(`|
/// |`\)`|`)`|
///
/// Additionally, trailing whitespace can be removed by preceding it with a `\`. The types of
/// whitespace that can be removed are newlines (`\n`), carriage returns (`\r`), spaces (` `),
/// and tabs (`\t`).
///
/// # Notes
///
/// - If tags are repeated in a style specifier, the value of the last tag takes precedence
/// - When nesting styled markup, styles of the parent will be applied unless overridden
/// - There is currently no way to remove text decorations from the children of nested markup
#[pyfunction]
fn colorize(source: &str) -> PyResult<String> {
    Ok(color::colorize(source).unwrap())
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "_antsi")]
fn antsi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(colorize, m)?)?;
    Ok(())
}
