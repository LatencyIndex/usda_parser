// i32::max is not const for some reason.
const fn max_i32(x: i32, y: i32) -> i32 {
    if x > y {
        x
    } else {
        y
    }
}

fn f64_to_i32(x: f64) -> Option<i32> {
    let min = i32::MIN as f64;
    let max = i32::MAX as f64;
    if (min..=max).contains(&x) {
        Some(x as i32)
    } else {
        None
    }
}

/// Number rounded to N significant digits.
/// Non-significant integrals become zero, and decimals are truncated.
/// ```
/// use riddle_of_protein::lsl::numeric::round_to_n_significant as rsig;
/// assert_eq!(rsig(123.456, 3), "123");
/// assert_eq!(rsig(1234.56, 3), "1230");
/// assert_eq!(rsig(1236.56, 3), "1240");
/// assert_eq!(rsig(0.0001234, 3), "0.000123");
/// assert_eq!(rsig(0.0001236, 3), "0.000124");
/// assert_eq!(rsig(0.00012, 3), "0.000120");
/// ```
pub fn round_to_n_significant(x: f64, significant_digits: i32) -> String {
    // Bigger than just the mantissa digits, because it includes
    // decimal leading zeros for numbers < 1, and trailing zeros for numbers > 1.
    const MAX_DIGITS: i32 = {
        // Conversion of f64::DIGITS from u32 to i32 is safe,
        // because no built-in numeric type has anywhere close to i32::MAX digits.
        const DIGITS: i32 = f64::DIGITS as i32;
        // If max is 1 * 10^N, that takes N+1 digits to represent:
        //      1 * 10^2 = 100, which has 3 digits (before the decimal, if any)
        const MAX_INTEGRALS: i32 = 1 + f64::MAX_10_EXP;
        // For negative N, if smallest (non-zero) is 1 * 10^N, that thakes -N digits to represent:
        //      1 * 10^-2 = 0.01, which has 2 digits (after the decimal)
        // Then we add the digits of the mantissa (which includes the +1 from the MAX_INTEGRALS formula).
        const MAX_DECIMALS: i32 = DIGITS - f64::MIN_10_EXP;
        max_i32(MAX_INTEGRALS, MAX_DECIMALS)
    };
    let significant_digits = significant_digits.clamp(1, MAX_DIGITS);

    // Position of first non-zero digit, indexed as follows around the decimal point:
    // ..[3][2][1][0].[-1][-2][-3]...
    let first_digit_pos = f64_to_i32(x.abs().log10().floor()).map(|i| i + 1);
    match first_digit_pos {
        Some(first_digit_pos) => {
            // Nb. of digits to retain in the fractional part of x.
            // Negative means that many digits in the integral part will get rounded.
            let decimals = significant_digits - first_digit_pos;
            if decimals >= 0 {
                // usize may be smaller than u32 on some platforms, so in principle this may fail.
                // But we don't care about those platforms, so a panic will do.
                let decimals = usize::try_from(decimals).unwrap();
                format!("{x:.decimals$}")
            } else {
                let precision = 10f64.powi(decimals.saturating_abs()).clamp(1.0, f64::MAX);
                let rounded = (x / precision).round() * precision;
                // Some digits that should be zero may not be due to float imprecision, so we zero them manually:
                let rounded = rounded.to_string();
                // Directly check for the minus sign, instead of checking if x is negative,
                // to avoid complications with negative vs. positive zero.
                // Should only be 0 or 1, depending on if there's a leading minus.
                // But should work in general, provided numbers < 1 don't omit the zero before the decimal point.
                let nb_leading_nondigits =
                    rounded.chars().take_while(|c| !c.is_ascii_digit()).count();
                // Following leading non-digits.
                let nb_leading_digits = rounded
                    .chars()
                    .skip(nb_leading_nondigits)
                    .take_while(char::is_ascii_digit)
                    .count();
                // Never fails because significant_digits is positive.
                let significant_digits = usize::try_from(significant_digits).unwrap();
                // Indexes of digits that should be turned to zero.
                let zero_range = (nb_leading_nondigits + significant_digits)
                    ..(nb_leading_nondigits + nb_leading_digits);
                let rounded: String = rounded
                    .char_indices()
                    .map(|(i, c)| if zero_range.contains(&i) { '0' } else { c })
                    .collect();
                rounded
            }
        }
        // Couldn't convert expression to i32 because it is NaN or out of range.
        // E.g. if we tried to do log10(0.0)
        None => {
            // Conversion is safe because significant_digits is guaranteed positive.
            let significant_digits = usize::try_from(significant_digits).unwrap();
            if x == 0.0 {
                format!("{x:.significant_digits$}")
            } else {
                // Nb. of integrals was not representable by an i32, and x != 0,
                // therefore it is either very small or very large, and the only reasonable
                // format remaining is scientific notation.
                // This should only happen for inf or NaN, so in practice this branch is unnecessary,
                // as inf and NaN string representation is the same in both formats.
                format!("{x:.significant_digits$e}")
            }
        }
    }
}
