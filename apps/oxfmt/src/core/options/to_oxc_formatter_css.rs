use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_css::{BraceStyle, CssFormatOptions, CssVariant, SingleQuote, TrailingCommas};

use super::super::oxfmtrc::{BraceStyleConfig, FormatConfig, TrailingCommaConfig};

/// Convert `FormatConfig` into `CssFormatOptions` for `oxc_formatter_css`.
///
/// Prettier's CSS languages consume the shared layout options plus
/// `singleQuote` and `trailingComma` (SCSS maps only).
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_css(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
    variant: CssVariant,
) -> CssFormatOptions {
    let mut options = CssFormatOptions { variant, ..CssFormatOptions::default() };
    options.apply_core(core_options);

    // [Prettier] singleQuote: boolean
    if let Some(single_quote) = config.single_quote {
        options.single_quote = SingleQuote::from(single_quote);
    }
    // [Prettier] trailingComma: "all" | "es5" | "none"
    // `all`/`es5` are indistinguishable for CSS (SCSS maps only check "not none")
    if let Some(trailing_comma) = config.trailing_comma {
        options.trailing_commas = match trailing_comma {
            TrailingCommaConfig::All | TrailingCommaConfig::Es5 => TrailingCommas::Always,
            TrailingCommaConfig::None => TrailingCommas::Never,
        };
    }
    // [Oxfmt] sortTailwindcss: collect `@apply` classes for batch sorting.
    // The sorter itself is JS-side, so this stays off in the pure Rust build
    // (classes would print as-is anyway, but skipping collection is cheaper).
    if config.brace_style.is_some_and(BraceStyleConfig::is_next_line) {
        options.brace_style = BraceStyle::NextLine;
    }
    #[cfg(feature = "napi")]
    {
        options.sort_tailwindcss = config.is_tailwind_enabled();
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brace_style() {
        for (json, expected) in [
            ("{}", BraceStyle::SameLine),
            (r#"{"braceStyle": "1tbs"}"#, BraceStyle::SameLine),
            (r#"{"braceStyle": "stroustrup"}"#, BraceStyle::SameLine),
            (r#"{"braceStyle": "allman"}"#, BraceStyle::NextLine),
        ] {
            let config: FormatConfig = serde_json::from_str(json).unwrap();
            let options =
                to_oxc_formatter_css(&config, CoreFormatOptions::default(), CssVariant::Css);
            assert_eq!(options.brace_style, expected, "{json}");
        }
    }
}
