use std::fmt::Display;

/// Appends a formatted term to a verbose output string, handling sign and formatting for display.
///
/// # Arguments
/// * `verbose_terms` - The string to which the formatted term will be appended.
/// * `c` - The coefficient (any type implementing `Display`).
/// * `value` - The value (any type implementing `Display`).
///
/// The function formats the term as `+/- coeff * value` and appends it to the string, handling the sign and spacing appropriately.
pub fn write_verbose_output<T: Display, V: Display>(verbose_terms: &mut String, c: &T, value: &V) {
    let coeff_str = c.to_string();
    let (is_negative, magnitude) = if coeff_str.starts_with('-') {
        (true, coeff_str.trim_start_matches('-'))
    } else {
        (false, coeff_str.as_str())
    };

    if verbose_terms.is_empty() {
        if is_negative {
            verbose_terms.push_str(&format!("-{} * {}", magnitude, value));
        } else {
            verbose_terms.push_str(&format!("{} * {}", magnitude, value));
        }
    } else {
        if is_negative {
            verbose_terms.push_str(&format!(" - {} * {}", magnitude, value));
        } else {
            verbose_terms.push_str(&format!(" + {} * {}", magnitude, value));
        }
    }
}
