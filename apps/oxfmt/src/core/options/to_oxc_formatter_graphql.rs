use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_graphql::{BraceStyle, BracketSpacing, GraphqlFormatOptions};

use super::super::oxfmtrc::{BraceStyleConfig, FormatConfig};

/// Convert `FormatConfig` into `GraphqlFormatOptions` for `oxc_formatter_graphql`.
///
/// Prettier's `graphql` language consumes only the shared layout options plus `bracketSpacing`.
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_graphql(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
) -> GraphqlFormatOptions {
    let mut options = GraphqlFormatOptions::default();
    options.apply_core(core_options);

    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        options.bracket_spacing = BracketSpacing::from(spacing);
    }
    if config.brace_style.is_some_and(BraceStyleConfig::is_next_line) {
        options.brace_style = BraceStyle::NextLine;
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
            (r#"{"braceStyle": "stroustrup"}"#, BraceStyle::SameLine),
            (r#"{"braceStyle": "allman"}"#, BraceStyle::NextLine),
        ] {
            let config: FormatConfig = serde_json::from_str(json).unwrap();
            let options = to_oxc_formatter_graphql(&config, CoreFormatOptions::default());
            assert_eq!(options.brace_style, expected, "{json}");
        }
    }
}
