use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, SubAssign};

/// A fixed point integer type with precision of four places past the decimal
///
/// This type stores floating point numbers as integers, multiplied by 10 000,
/// to keep 4 decimal places. Any additional precision is lost after the conversion to Amount.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Amount(u64);

impl Amount {
    /// Maximum value am `Amount` can represent
    /// ```
    /// use transactions_lib::amount::Amount;
    ///
    /// assert_eq!(Amount::MAX.to_string(), "1844674407370955.1615");
    pub const MAX: Amount = Amount(u64::MAX);

    /// Returns a zero value
    /// ```
    /// use transactions_lib::amount::Amount;
    ///
    /// assert_eq!(Amount::zero().to_string(), "0");
    pub fn zero() -> Amount {
        Amount(0)
    }

    /// Converts an f64 to Amount. Any additional precision after the four places past the decimal will be truncated
    /// ```
    /// use transactions_lib::amount::Amount;
    ///
    /// assert_eq!(Amount::from_f64(123.456).to_string(), "123.456");
    /// assert_eq!(Amount::from_f64(123.45678).to_string(), "123.4567");
    /// assert_eq!(Amount::from_f64(-123.45678).to_string(), "0");
    /// ```
    pub fn from_f64(real_value: f64) -> Amount {
        Amount((real_value * 10_000.0) as u64)
    }

    /// Parses string to Amount. Any additional precision after the four places past the decimal will be truncated
    /// ```
    /// use transactions_lib::amount::Amount;
    ///
    /// assert_eq!(Amount::parse("123.456").unwrap().to_string(), "123.456");
    /// assert_eq!(Amount::parse("0.0001").unwrap().to_string(), "0.0001");
    /// assert_eq!(Amount::parse("1844674407370955.1615").unwrap().to_string(), "1844674407370955.1615");
    /// // Numbers out of range are truncated
    /// assert_eq!(Amount::parse("1844674407370955.1616").unwrap().to_string(), "1844674407370955.1615");
    /// assert_eq!(Amount::parse("991844674407370955.9999").unwrap().to_string(), "1844674407370955.1615");
    /// assert_eq!(Amount::parse("0.0000001").unwrap().to_string(), "0");
    /// ```
    pub fn parse(str: &str) -> Option<Amount> {
        Some(Amount::from_f64(str.parse::<f64>().ok()?))
    }
}

/// Makes it possible to use `+=` operator for `Amount`s
/// ```
/// use transactions_lib::amount::Amount;
///
/// let mut amount = Amount::parse("0.456").unwrap();
/// amount += Amount::parse("123").unwrap();
/// assert_eq!(amount.to_string(), "123.456");
/// ```
impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// Makes it possible to use `-=` operator for `Amount`s
/// ```
/// use transactions_lib::amount::Amount;
///
/// let mut amount = Amount::parse("123.456").unwrap();
/// amount -= Amount::parse("123").unwrap();
/// assert_eq!(amount.to_string(), "0.456");
/// ```
impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// Format the type.
/// If there are no digits past the decimal, only the integer part will be written:
/// ```
/// use transactions_lib::amount::Amount;
///
/// assert_eq!(Amount::parse("123.0000").unwrap().to_string(), "123");
/// ```
impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut remainding_number = self.0;
        let int_part = remainding_number / 10_000;
        write!(f, "{}", int_part)?;

        remainding_number -= int_part * 10_000;
        if remainding_number == 0 {
            // write only the integer part if there are no fraction digits
            return Ok(());
        }
        write!(f, ".")?;

        // print the digits one by one and decrease the `remainding_number` by the printed value
        let mut remainding_divident = 1000;
        while remainding_number > 0 {
            let fraction_digit = remainding_number / remainding_divident;
            write!(f, "{}", fraction_digit)?;
            remainding_number -= fraction_digit * remainding_divident;
            remainding_divident /= 10;
        }

        Ok(())
    }
}
