//! Static string table for the tray (pt-BR, English fallback). No i18n framework on purpose.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    PtBr,
    En,
}

impl Locale {
    /// Maps a BCP 47 / POSIX locale tag; anything that is not Portuguese is English.
    pub fn parse(tag: &str) -> Self {
        if tag.trim().to_ascii_lowercase().starts_with("pt") {
            Self::PtBr
        } else {
            Self::En
        }
    }

    pub fn system() -> Self {
        sys_locale::get_locale().map_or(Self::En, |l| Self::parse(&l))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    OpenApp,
    CompressNow,
    Quit,
    Tooltip,
}

/// Rows follow `Key` declaration order; columns are `[pt-BR, en]`.
pub const KEY_COUNT: usize = 4;
const TABLE: [[&str; 2]; KEY_COUNT] = [
    ["Abrir Disk Steward", "Open Disk Steward"],
    ["Comprimir agora", "Compress now"],
    ["Sair", "Quit"],
    ["Disk Steward: {free} livres", "Disk Steward: {free} free"],
];

pub fn tr(loc: Locale, key: Key) -> &'static str {
    TABLE[key as usize][loc as usize]
}

/// Substitutes `{name}` placeholders.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    args.iter().fold(template.to_string(), |s, (k, v)| {
        s.replace(&format!("{{{k}}}"), v)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_complete_and_accented() {
        assert_eq!(Key::Tooltip as usize, KEY_COUNT - 1);
        assert!(TABLE.iter().flatten().all(|s| !s.is_empty()));
        assert_eq!(
            tr(Locale::PtBr, Key::Tooltip),
            "Disk Steward: {free} livres"
        );
        assert_eq!(
            fill(tr(Locale::En, Key::Tooltip), &[("free", "75 GB")]),
            "Disk Steward: 75 GB free"
        );
    }

    #[test]
    fn locale_parsing_falls_back_to_en() {
        assert_eq!(Locale::parse("pt-BR"), Locale::PtBr);
        assert_eq!(Locale::parse("pt_PT.UTF-8"), Locale::PtBr);
        assert_eq!(Locale::parse("es-MX"), Locale::En);
        assert_eq!(Locale::parse(""), Locale::En);
    }
}
