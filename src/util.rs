use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

fn adjust_indices(
    indices: &[(u32, u32)],
    original_width: u32,
    truncation_size: u32,
) -> Vec<(u32, u32)> {
    if indices.is_empty() || truncation_size >= original_width {
        return indices.to_vec();
    }

    let offset = original_width.saturating_sub(truncation_size);

    indices
        .iter()
        .filter_map(|range| {
            let x = range.0.saturating_sub(offset);
            let y = range.1.saturating_sub(offset);
            if x == 0 && y == 0 {
                None
            } else {
                Some((x, y))
            }
        })
        .collect()
}

pub fn align_str(
    s: &str,
    indices: &[(u32, u32)],
    max_width: u32,
    replacement_text: &str,
    hscroll_offset: u32,
) -> (String, Vec<(u32, u32)>) {
    if s.is_empty() {
        return (String::new(), vec![]);
    }

    let mut current_length = s.len() as u32;
    let mut current_string = String::from(s);
    let mut current_width = current_string.width() as u32;
    let mut current_indices: Vec<(u32, u32)> = indices.to_vec();

    if current_width <= max_width {
        return (current_string.to_string(), current_indices);
    }

    let replacement_width = replacement_text.width() as u32;

    if indices.is_empty() {
        let (truncated_string, _) =
            current_string.unicode_truncate(max_width.saturating_sub(replacement_width) as usize);

        return (
            format!("{}{}", truncated_string, replacement_text),
            current_indices,
        );
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

    let trailing_width = (current_string.width() as u32)
        .saturating_sub(max_match.saturating_add(1))
        .saturating_sub(hscroll_offset);

    if trailing_width > 0 {
        let truncation_size = current_width
            .saturating_sub(trailing_width)
            .saturating_sub(replacement_width)
            .max(max_width.saturating_sub(replacement_width))
            as usize;

        let (truncated_string, _) = current_string.unicode_truncate(truncation_size);

        current_string = format!("{}{}", truncated_string, replacement_text);
        current_length = current_string.len() as u32;
        current_width = current_string.width() as u32;

        if current_width <= max_width {
            return (current_string, current_indices);
        }
    }

    let leading_width = min_match;

    if leading_width > 0 {
        let truncation_size = current_width
            .saturating_sub(leading_width)
            .saturating_sub(replacement_width)
            .max(max_width.saturating_sub(replacement_width));
        let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size as usize);

        current_string = format!("{}{}", replacement_text, truncated_string);
        current_indices = adjust_indices(
            &current_indices,
            // current_width,
            current_length,
            current_string.len() as u32,
        );
        current_length = current_string.len() as u32;
        current_width = current_string.width() as u32;

        if current_width <= max_width {
            return (current_string, current_indices);
        }
    }

    let truncation_size = max_width.saturating_sub(replacement_width);
    let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size as usize);

    current_string = format!("{}{}", replacement_text, truncated_string);
    current_indices = adjust_indices(
        &current_indices,
        current_length,
        current_string.len() as u32,
    );

    (current_string, current_indices)
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
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "Code/linux/scripts/dummy-tools/dummy-pl…");
        assert_eq!(result.1, vec![]);
    }

    #[test]
    fn simple_with_late_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(65, 71)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0.len(), 42);
        assert_eq!(result.0, "…mmy-plugin-dir/include/plugin-version.h");
        assert_eq!(result.1, vec![(35, 41)]);
    }

    #[test]
    fn simple_with_early_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11)], 40, "…", 10);
        assert_eq!(result.1, vec![(5, 11)]);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "Code/linux/scripts/dummy-tools/dummy-pl…");
    }

    #[test]
    fn simple_with_mid_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(30, 38)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0.len(), 44);
        assert_eq!(result.0, "…/scripts/dummy-tools/dummy-plugin-dir/…");
        assert_eq!(result.1, vec![(23, 31)]);
    }

    #[test]
    fn simple_with_multiple_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11), (60, 67)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0.len(), 42);
        assert_eq!(result.0, "…mmy-plugin-dir/include/plugin-version.h");
        assert_eq!(result.1, vec![(30, 37)]);
    }
}
