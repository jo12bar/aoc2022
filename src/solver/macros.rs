macro_rules! challenge_solver_test_boilerplate {
    {
        $challenge_solver:expr;
        $sample_input:expr => {
            a as $res_type_a:ty : $res_a:expr,
            b as $res_type_b:ty : $res_b:expr $(,)?
        }
        $($other_tests:tt)*
    } => {
        #[cfg(test)]
        mod tests {
            use super::*;
            use $crate::solver::ChallengeSolver;
            use std::io::Cursor;

            const SAMPLE_INPUT: &str = $sample_input;

            #[test]
            fn test_a() -> color_eyre::Result<()> {
                color_eyre::install()?;
                let mut input = Cursor::new(SAMPLE_INPUT);
                let mut solver = $challenge_solver;

                let res = solver.solve_a(&mut input)?;

                let res = res.downcast_ref::<$res_type_a>().ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Could not cast challenge solver result to {}",
                        stringify!($res_type_a)
                    )
                })?;

                assert_eq!(res, &$res_a);

                Ok(())
            }

            #[test]
            fn test_b() -> color_eyre::Result<()> {
                color_eyre::install()?;
                let mut input = Cursor::new(SAMPLE_INPUT);
                let mut solver = $challenge_solver;

                let res = solver.solve_b(&mut input)?;

                let res = res.downcast_ref::<$res_type_b>().ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Could not cast challenge solver result to {}",
                        stringify!($res_type_b)
                    )
                })?;

                assert_eq!(res, &$res_b);

                Ok(())
            }

            $($other_tests)*
        }
    };

    {
        $challenge_solver:expr;
        $sample_input:expr => {
            b as $res_type_b:ty : $res_b:expr,
            a as $res_type_a:ty : $res_a:expr $(,)?
        }
        $($other_tests:tt)*
    } => {
        $crate::solver::macros::challenge_solver_test_boilerplate!{
            $challenge_solver;
            $sample_input => {
                a as $res_type_a: $res_a,
                b as $res_type_b: $res_b,
            }
            $($other_tests)*
        }
    };
}
pub(super) use challenge_solver_test_boilerplate;
