//! The catalog of TypeScript/JavaScript behaviors that do not carry over to Rust as-is.

use serde::Serialize;

/// A construct whose JS semantics differ from the obvious Rust translation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Risk {
    StringOffsets,
    RegexSyntax,
    DynamicRegex,
    DynamicCode,
    DynamicTyping,
    OrderedCollections,
    Exceptions,
    Async,
    Generators,
    ClassInheritance,
    Reflection,
    Callbacks,
    NumberSemantics,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn weight(self) -> f64 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 3.0,
            Self::High => 6.0,
        }
    }
}

impl Risk {
    pub fn severity(self) -> Severity {
        match self {
            Self::StringOffsets | Self::RegexSyntax | Self::DynamicRegex | Self::DynamicCode => {
                Severity::High
            }
            Self::DynamicTyping
            | Self::OrderedCollections
            | Self::Async
            | Self::Generators
            | Self::ClassInheritance
            | Self::Reflection
            | Self::Callbacks => Severity::Medium,
            Self::Exceptions | Self::NumberSemantics | Self::Json => Severity::Low,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::StringOffsets => "String offsets (UTF-16 vs UTF-8)",
            Self::RegexSyntax => "Regex features the `regex` crate lacks",
            Self::DynamicRegex => "Regexes built at runtime",
            Self::DynamicCode => "Dynamic code (eval, new Function, computed import)",
            Self::DynamicTyping => "Dynamic typing (`any`, `in` checks)",
            Self::OrderedCollections => "Insertion-ordered maps and object keys",
            Self::Exceptions => "Exceptions",
            Self::Async => "Async and promises",
            Self::Generators => "Generators",
            Self::ClassInheritance => "Class inheritance",
            Self::Reflection => "Reflection and prototypes",
            Self::Callbacks => "Callback-shaped APIs",
            Self::NumberSemantics => "Number semantics",
            Self::Json => "JSON round-trips",
        }
    }

    pub fn guidance(self) -> &'static str {
        match self {
            Self::StringOffsets => {
                "JS indexes strings in UTF-16 code units, Rust in UTF-8 bytes. Offsets that cross \
                 the boundary must be converted, or non-ASCII text gets wrong ranges silently."
            }
            Self::RegexSyntax => {
                "Lookaround, backreferences and the `y` flag are not in the `regex` crate. Use \
                 `fancy-regex` for those patterns, or rewrite them."
            }
            Self::DynamicRegex => {
                "Patterns come from data or config in JS syntax. Rust regex syntax differs, so \
                 user-visible behavior can change. Pin it with tests on real patterns."
            }
            Self::DynamicCode => {
                "eval, new Function and computed imports have no Rust equivalent. Keep this part \
                 in JS or replace it with an explicit registry."
            }
            Self::DynamicTyping => {
                "Values whose shape is checked at runtime need an explicit Rust enum or \
                 `serde_json::Value`."
            }
            Self::OrderedCollections => {
                "JS Map, Set and object keys keep insertion order. `HashMap` does not: use \
                 `indexmap` wherever order reaches output."
            }
            Self::Exceptions => {
                "Thrown errors become `Result`. Errors crossing the bridge must map back to the \
                 same JS error shape."
            }
            Self::Async => {
                "Promises become futures. Bridge with napi-rs `async fn` or `wasm-bindgen-futures`, \
                 or keep the async shell in JS and port the sync core."
            }
            Self::Generators => {
                "Lazy generators become iterators, or return a collected Vec across the bridge."
            }
            Self::ClassInheritance => {
                "Class hierarchies become traits plus composition; `instanceof` checks become enums."
            }
            Self::Reflection => {
                "Proxy, Reflect, Symbol and prototype tricks need a redesign, not a translation."
            }
            Self::Callbacks => {
                "Function-typed parameters and methods cross the bridge on every call. Keep \
                 boundaries coarse: pass data in, get data out."
            }
            Self::NumberSemantics => {
                "JS numbers are f64; bitwise ops truncate to i32; parseInt is lenient. Pick Rust \
                 integer types deliberately."
            }
            Self::Json => {
                "JSON.stringify key order and number formatting must match `serde_json` output if \
                 the result is user-visible."
            }
        }
    }
}

/// Whether a JS regex uses features the `regex` crate does not support.
pub fn regex_needs_fancy(pattern: &str, flags: &str) -> bool {
    if flags.contains('y') {
        return true;
    }
    if ["(?=", "(?!", "(?<=", "(?<!", "\\k<"]
        .iter()
        .any(|needle| pattern.contains(needle))
    {
        return true;
    }
    let bytes = pattern.as_bytes();
    let mut in_class = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                if !in_class && bytes.get(i + 1).is_some_and(|b| (b'1'..=b'9').contains(b)) {
                    return true;
                }
                i += 1;
            }
            b'[' => in_class = true,
            b']' => in_class = false,
            _ => {}
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::regex_needs_fancy;

    #[test]
    fn detects_unsupported_regex_features() {
        assert!(regex_needs_fancy("(?<=a)b", ""));
        assert!(regex_needs_fancy("(a)\\1", ""));
        assert!(regex_needs_fancy("a", "y"));
        assert!(!regex_needs_fancy("[\\w\\d]+?", "g"));
        assert!(!regex_needs_fancy("(?<name>x)", "d"));
        assert!(!regex_needs_fancy("[\\1]", ""));
    }
}
