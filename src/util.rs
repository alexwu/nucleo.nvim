use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

fn adjust_indices(
    indices: &[(usize, usize)],
    original_width: usize,
    truncation_size: usize,
) -> Vec<(usize, usize)> {
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
    indices: &[(usize, usize)],
    max_width: usize,
    replacement_text: &str,
    hscroll_offset: usize,
) -> (String, Vec<(usize, usize)>) {
    if s.is_empty() {
        return (String::new(), vec![]);
    }

    let mut current_string = String::from(s);
    let mut current_width = current_string.width();
    let mut current_indices: Vec<(usize, usize)> = indices.to_vec();

    dbg!(current_width);
    if current_width <= max_width {
        return (current_string.to_string(), current_indices);
    }

    let replacement_width = replacement_text.width();
    if indices.is_empty() {
        let (truncated_string, _) = current_string.unicode_truncate(max_width - replacement_width);
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
            return (current_string, current_indices);
        }
    }

    let leading_width = min_match + 1;
    dbg!(leading_width);

    if leading_width > 0 {
        let truncation_size = current_width
            .saturating_sub(leading_width)
            .saturating_sub(replacement_width)
            .max(max_width.saturating_sub(replacement_width));
        let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size);

        // let offset = current_width.saturating_sub(truncation_size);
        current_indices = adjust_indices(&current_indices, current_width, truncation_size);
        current_string = format!("{}{}", replacement_text, truncated_string);
        current_width = current_string.width();
        dbg!(current_width);

        if current_width <= max_width {
            // current_indices = current_indices
            //     .iter()
            //     .filter_map(|range| {
            //         let x = range.0.saturating_sub(offset);
            //         let y = range.1.saturating_sub(offset);
            //         if x == 0 && y == 0 {
            //             None
            //         } else {
            //             Some((x, y))
            //         }
            //     })
            //     .collect();
            return (current_string, current_indices);
        }
    }

    let truncation_size = max_width.saturating_sub(replacement_width);
    let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size);
    current_indices = adjust_indices(&current_indices, current_width, truncation_size);
    // let offset = current_width.saturating_sub(truncation_size);

    // current_indices = current_indices
    //     .iter()
    //     .filter_map(|range| {
    //         let x = range.0.saturating_sub(offset);
    //         let y = range.1.saturating_sub(offset);
    //         if x == 0 && y == 0 {
    //             None
    //         } else {
    //             Some((x, y))
    //         }
    //     })
    //     .collect();
    (
        format!("{}{}", replacement_text, truncated_string),
        current_indices,
    )
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

        let result = align_str(&fixture, &[(65, 72)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "…mmy-plugin-dir/include/plugin-version.h");
        assert_eq!(result.1, vec![(32, 39)]);
    }

    #[test]
    fn simple_with_early_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "Code/linux/scripts/dummy-tools/dummy-pl…");
        assert_eq!(result.1, vec![(5, 11)]);
    }

    #[test]
    fn simple_with_mid_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(30, 38)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "…x/scripts/dummy-tools/dummy-plugin-dir…");
        assert_eq!(result.1, vec![(21, 29)]);
    }

    #[test]
    fn simple_with_multiple_indices() {
        let fixture =
            "Code/linux/scripts/dummy-tools/dummy-plugin-dir/include/plugin-version.h".to_string();

        let result = align_str(&fixture, &[(5, 11), (60, 67)], 40, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "…mmy-plugin-dir/include/plugin-version.h");
        assert_eq!(result.1, vec![(26, 33)]);
    }
}
