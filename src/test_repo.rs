use rand::Rng;

pub fn generate_question<R: Rng>(rng: &mut R) -> (String, i64) {
    loop {
        let a = rng.gen_range(2..=100);
        let b = rng.gen_range(2..=100);
        let op = rng.gen_range(0..4);

        match op {
            0 => {
                // Addition
                return (format!("{} + {}", a, b), a + b);
            }
            1 => {
                // Subtraction, ensure result is non-negative
                if a >= b {
                    return (format!("{} - {}", a, b), a - b);
                }
            }
            2 => {
                // Multiplication
                let b = rng.gen_range(2..=20);
                return (format!("{} x {}", a, b), a * b);
            }
            3 => {
                // Division, ensure result is an integer
                let divisor = rng.gen_range(3..=30); // Limit divisor to ensure result is within 1-100
                let quotient = rng.gen_range(3..=30);
                let dividend = divisor * quotient;
                if dividend <= 100 {
                    return (format!("{} ÷ {}", dividend, divisor), quotient);
                }
            }
            _ => unreachable!(),
        }
    }
}
