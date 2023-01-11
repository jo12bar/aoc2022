macro_rules! challenge_solver_test_boilerplate {
    {
        $challenge_solver:expr;
        $sample_input:expr => {
            a as $res_type_a:ty : $res_a:expr,
            b as $res_type_b:ty : $res_b:expr $(,)?
        }
    } => {
        #[cfg(test)]
        mod tests {
            use super::*;
            use $crate::solver::ChallengeSolver;
            use std::io::Cursor;

            const SAMPLE_INPUT: &str = $sample_input;

            #[test]
            fn test_a() -> color_eyre::Result<()> {
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
        }
    };

    {
        $challenge_solver:expr;
        $sample_input:expr => {
            b as $res_type_b:ty : $res_b:expr,
            a as $res_type_a:ty : $res_a:expr $(,)?
        }
    } => {
        $crate::solver::macros::challenge_solver_test_boilerplate!{
            $challenge_solver;
            $sample_input => {
                a as $res_type_a: $res_a,
                b as $res_type_b: $res_b,
            }
        }
    };
}
pub(super) use challenge_solver_test_boilerplate;
