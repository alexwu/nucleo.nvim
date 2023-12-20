use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

pub fn align_str(
    s: &str,
    indices: &[(usize, usize)],
    max_width: usize,
    replacement_text: &str,
    hscroll_offset: usize,
) -> String {
    if s.is_empty() {
        return String::new();
    }

    let mut current_string = String::from(s);
    let mut current_width = current_string.width();
    dbg!(current_width);
    if current_width <= max_width {
        return current_string.to_string();
    }

    let replacement_width = replacement_text.width();
    if indices.is_empty() {
        let (truncated_string, _) = current_string.unicode_truncate(max_width - replacement_width);
        return format!("{}{}", truncated_string, replacement_text);
    }

    let min_match = indices
        .iter()
        .map(|index| index.0)
        .min()
        .expect("Somehow unable to find min_match");
    let max_match = indices
        .iter()
        .map(|index| index.1)
        .max()
        .expect("somehow unable to find max_match");

    let trailing_width = current_string
        .width()
        .saturating_sub(max_match)
        .saturating_sub(hscroll_offset);
    dbg!(trailing_width);

    if trailing_width > 0 {
        let (truncated_string, _) = current_string.unicode_truncate(
            current_width
                .saturating_sub(trailing_width)
                .saturating_sub(replacement_width)
                .max(max_width.saturating_sub(replacement_width)),
        );

        current_string = format!("{}{}", truncated_string, replacement_text);
        current_width = current_string.width();

        dbg!(current_width);
        if current_width <= max_width {
            return current_string;
        }
    }

    let leading_width = min_match + 1;
    dbg!(leading_width);

    if leading_width > 0 {
        let (truncated_string, _) = current_string.unicode_truncate_start(
            current_width
                .saturating_sub(leading_width)
                .saturating_sub(replacement_width)
                .max(max_width.saturating_sub(replacement_width)),
        );

        current_string = format!("{}{}", replacement_text, truncated_string);
        current_width = current_string.width();
        dbg!(current_width);

        if current_width <= max_width {
            return current_string;
        }
    }

    let (truncated_string, _) =
        current_string.unicode_truncate_start(max_width.saturating_sub(replacement_width));

    format!("{}{}", replacement_text, truncated_string)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[], 40, "…", 10);
        assert_eq!(result.width(), 40);
        assert_eq!(result, "Code/linux/scripts/dummy-tools/dummy-pl…");
    }

    #[test]
    fn simple_with_late_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(65, 72)], 40, "…", 10);
        assert_eq!(result.width(), 40);
        assert_eq!(result, "…mmy-plugin-dir/include/plugin-version.h");
    }

    #[test]
    fn simple_with_early_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11)], 40, "…", 10);
        assert_eq!(result.width(), 40);
        assert_eq!(result, "Code/linux/scripts/dummy-tools/dummy-pl…");
    }

    #[test]
    fn simple_with_mid_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(30, 38)], 40, "…", 10);
        assert_eq!(result.width(), 40);
        assert_eq!(result, "…x/scripts/dummy-tools/dummy-plugin-dir…");
    }

    #[test]
    fn simple_with_multiple_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11), (60, 67)], 40, "…", 10);
        assert_eq!(result.width(), 40);
        assert_eq!(result, "…mmy-plugin-dir/include/plugin-version.h");
    }
}
