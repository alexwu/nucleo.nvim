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

    // dbg!(original_width);
    // dbg!(truncation_size);
    let offset = original_width.saturating_sub(truncation_size);
    // dbg!(offset);
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

    // dbg!(current_width);
    if current_width <= max_width {
        return (current_string.to_string(), current_indices);
    }

    let replacement_width = replacement_text.width() as u32;
    // dbg!(replacement_width);
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
    // dbg!(leading_width);

    if leading_width > 0 {
        let truncation_size = current_width
            .saturating_sub(leading_width)
            .saturating_sub(replacement_width)
            .max(max_width.saturating_sub(replacement_width));
        let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size as usize);

        // let offset = current_width.saturating_sub(truncation_size);
        current_string = format!("{}{}", replacement_text, truncated_string);
        current_indices = adjust_indices(
            &current_indices,
            // current_width,
            current_length,
            current_string.len() as u32,
        );
        current_length = current_string.len() as u32;
        current_width = current_string.width() as u32;
        // dbg!(current_width);

        if current_width <= max_width {
            return (current_string, current_indices);
        }
    }

    let truncation_size = max_width.saturating_sub(replacement_width);
    let (truncated_string, _) = current_string.unicode_truncate_start(truncation_size as usize);

    current_string = format!("{}{}", replacement_text, truncated_string);
    current_indices = adjust_indices(&current_indices, current_length, current_string.len() as u32);

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

        let result = align_str(&fixture, &[(65, 71)], 40, "...", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.0, "...y-plugin-dir/include/plugin-version.h");
        assert_eq!(result.1, vec![(33, 39)]);
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
        assert_eq!(vec![(21, 29)], result.1);
        assert_eq!(result.0, "…x/scripts/dummy-tools/dummy-plugin-dir…");
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
    #[test]
    fn really_long_with_indices_end() {
        let fixture =
            "spec/fixtures/vcr_cassettes/EditOrder_OrderProcessor_V2_Processor_CancelsOriginalOrder_OrderCanceler/OrderProcessorRemoveLineItemsOnCancel_flipper_is_enabled/AND_editing_with_do_not_return_stock_/adds_refund_line_items_to_the_cancel_args_in_order_to_remove_items_from_the_order_on_cancel.yml".to_string();

        let result = align_str(&fixture, &[(5, 11), (288, 291)], 110, "…", 10);
        assert_eq!(result.0.width(), 40);
        assert_eq!(result.1, vec![(288, 291)]);
        assert_eq!(result.0, "");
    }
}
