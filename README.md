# Antsi

A quick and user-friendly way to style your text using ANSI codes.

Built around a delightfully simple markup language, there's no messing about with codes or style resets. All the ANSI
code conversion, handling of overlapping styles, and terminal support is automatically handled for you.

## Usage

You can either download wheels from the [Releases][latest-release] tab or install it from [PyPI][pypi]:

```shell
pip install antsi
```

The function you'll most likely care about is `colorize`. Colorize is responsible for converting any styled markup
within the provided source to ANSI escape sequences. The [styled markup](#styled-markup) syntax can be found below.

You can use the following code to test that it's working as expected.

```python
import antsi

print(antsi.colorize("Hello [fg:green](there!)"))
print(antsi.colorize("It looks like [fg:black;bg:white;deco:bold,blink](antsi) is working!"))
```

If you're planning on doing any string manipulation or templating with styled markup, you'll want to `escape` the
substituted text to prevent any unexpected colors or errors. The `escape` function ensures that any control characters
are properly escaped.

## Styled Markup

Styled markup is a decidedly simple language that is make for ease of use and readability. There's no unruly sequences
of numbers and characters that you'll forget what they mean in a week, just words and a couple memorable abbreviations.

Styled markup can appear anywhere within a piece of text. It can even be nested infinitely! The only requirements are
that it start with a [style specifier](#style-specifiers) and is followed by some [content](#content). It will always be
in the form:

```text
[ <style specifier(s)> ]( <content> )
```

For example, `Hello [fg:green](there), user!` which will make only "there" be colored green. Everything else will be the
default text color.

### Content

The content following a style specifier must always be wrapped in parentheses and is whitespace sensitive. Any
whitespace within the content will be emitted as-is unless [escaped](#escape-sequences) using a backslash
(<code>&bsol;</code>).

> [!IMPORTANT]
>
> Putting any character between the style specifier and content is explicitly disallowed.

The content can contain any characters, and it can even contain other styled markup! If you want to use any square
brackets or parentheses in the content, they must be [escaped](#escape-sequences).

> [!TIP]
>
> When nesting styled markup, the styles of the parent markup will also be applied unless explicitly overridden.
> However, there is currently no way to remove text decorations from the children of nested markup.

### Style Specifiers

Style specifiers are surrounded by square brackets (`[]`) and contain the directives for applying style to the content
immediately following them. They are essentially a list of key-value pairs, where the key denotes the type of style.

Examples:

| Style Specifier                 | Meaning                                                    |
|---------------------------------|------------------------------------------------------------|
| `[fg:red]`                      | Red foreground                                             |
| `[bg:blue]`                     | Blue background                                            |
| `[deco:bold]`                   | Bold text                                                  |
| `[deco:bold,blink]`             | Bold and blinking                                          |
| `[fg:red;bg:white]`             | Red foreground and white background                        |
| `[bg:blue;fg:white;deco:bold]`  | White foreground, blue background, and bold                |
| `[deco:bold,underline;fg:cyan]` | Cyan foreground, bold, and underlined                      |
| `[fg:red;bg:white;fg:blue]`     | Blue foreground and white background (last specifier wins) |

> [!IMPORTANT]
>
> If any tags are repeated in the specifier, the value of the last tag takes precedence. This means that tags which can
> accept multiple values (i.e. `deco`) are *not* merged.

As show above, there are currently three specifiers that are allowed: [`fg`](#foreground-color-fg) (
foreground), [`bg`](#background-color-bg) (background), and [`deco`](#text-decoration-deco) (decoration).

#### Foreground color (`fg`)

Format: `fg:<color>`

Changes the color of the text itself. Currently only the standard color pallet is implemented, providing 8 colors with a
standard and a bright variant. The bright variant can be chosen by prefixing the color with `bright-`.

> [!TIP]
>
> Colors will appear differently depending on the terminal being used.

| Color   | Standard Code | Bright Code      |
|---------|---------------|------------------|
| Default | `default`     | N/A              |
| Black   | `black`       | `bright-black`   |
| Red     | `red`         | `bright-red`     |
| Green   | `green`       | `bright-green`   |
| Yellow  | `yellow`      | `bright-yellow`  |
| Blue    | `blue`        | `bright-blue`    |
| Magenta | `magenta`     | `bright-magenta` |
| Cyan    | `cyan`        | `bright-cyan`    |
| White   | `white`       | `bright-white`   |

#### Background color (`bg`)

Format: `bg:<color>`

Changes the color of the text background. It accepts the same colors as [`fg`](#foreground-color-fg).

#### Text decoration (`deco`)

Format: `deco:<decoration>,[<decoration>...]`

Applies additional text decorations like bolding, dimming, blinking, etc. Unlike the foreground and background, multiple
text decorations can be applied at the same time using a comma-separated list.

> [!TIP]
>
> Support for decorations may differ between terminals.

| Decoration    | Code(s)                           |
|---------------|-----------------------------------|
| Bold          | `bold`                            |
| Dim           | `dim`, `faint`                    |
| Italic        | `italic`                          |
| Underline     | `underline`                       |
| Fast Blink    | `fast-blink`, `blink-fast`        |
| Slow Blink    | `slow-blink`, `blink-slow`        |
| Invert        | `invert`, `reverse`               |
| Hide          | `hide`, `conceal`                 |
| Strikethrough | `strike-through`, `strikethrough` |

### Escape Sequences

There are a handful of control characters that must be escaped to include them anywhere in your text. This includes
text outside styled markup.

| Character           | Sequence                  |
|---------------------|---------------------------|
| <code>&bsol;</code> | <code>&bsol;&bsol;</code> |
| `[`                 | <code>&bsol;&lsqb;</code> |
| `]`                 | <code>&bsol;&rsqb;</code> |
| `(`                 | <code>&bsol;&lpar;</code> |
| `)`                 | <code>&bsol;&rpar;</code> |

Beyond these characters, you can also escape any whitespace (i.e. spaces, tabs, newlines, and carriage returns) to make
writing multi-line text easier. All you need to do is prefix it with a backslash (<code>&bsol;</code>).

## Contributing

Any and all contributions are welcome! Some ideas if you can't think of anything:

- Improve the documentation
- Add some functionality that might be missing
- Report (or even fix) any bugs

If you don't have time to contribute yourself but still wish to support the project, [sponsorship][sponsorship] would be
greatly appreciated!

[latest-release]: https://github.com/akrantz01/antsi/releases
[pypi]: https://pypi.org/p/antsi
[sponsorship]: https://github.com/sponsors/akrantz01
