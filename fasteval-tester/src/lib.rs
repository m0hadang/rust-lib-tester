use std::collections::BTreeMap;
use fasteval::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ez_test() {
        let expr_str =
            "1+2*3/4^5%6 + log(100K) + log(e(),100) + [3*(3-3)/3] + (2<3) && 1.23";
        //    |            |      |    |   |          |               |   |
        //    |            |      |    |   |          |               |   boolean logic with short-circuit support
        //    |            |      |    |   |          |               comparisons
        //    |            |      |    |   |          square-brackets act like parenthesis
        //    |            |      |    |   built-in constants: e(), pi()
        //    |            |      |    'log' can take an optional first 'base' argument, defaults to 10
        //    |            |      numeric literal with suffix: n, µ, m, K, M, G, T
        //    |            many built-in functions: print, int, ceil, floor, abs, sign, log, round, min, max, sin, asin, ...
        //    standard binary operators
        let res = ez_eval(expr_str, &mut EmptyNamespace);
        if let Ok(val) = res {
            assert_eq!(val, 1.23);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_name_space() {
        // !! only can use f64 type
        let mut name_space = |name: &str, args: Vec<f64>| -> Option<f64> {
            let data: [f64; 3] = [11.1, 22.2, 33.3];
            match name {
                // Custom constants/variables:
                "x" => Some(3.0),
                "y" => Some(4.0),
                // Custom function:
                "sum" => Some(args.into_iter().fold(0.0, |s, f| s + f)),
                // Custom array-like objects:
                // The `args.get...` code is the same as:
                //     mydata[args[0] as usize]
                // ...but it won't panic if either index is out-of-bounds.
                "data" => args
                    .get(0)
                    .and_then(
                        |idx| data.get(*idx as usize).copied()),
                // A wildcard to handle all undefined names:
                _ => None,
            }
        };
        let custrom_function = "sum(x^2, y^2)^0.5 + data[0]";
        let Ok(val) = ez_eval(custrom_function, &mut name_space) else {
            assert!(false);
            return;
        };
        assert_eq!(val, 16.1);
    }

    #[test]
    fn test_btree_map() {
        {
            let mut map: BTreeMap<&'static str, f64> = BTreeMap::new();
            map.insert("x", 2.0);
            let Ok(val) = ez_eval("x * (x + 1)", &mut map) else {
                assert!(false);
                return;
            };
            assert_eq!(val, 6.0);
        }
        {
            let mut map: BTreeMap<String, f64> = BTreeMap::new();
            map.insert("x".to_string(), 2.0);
            let Ok(val) = ez_eval("x * (x + 1)", &mut map) else {
                assert!(false);
                return;
            };
            assert_eq!(val, 6.0);
        }
    }

    #[test]
    fn test_slab() {
        slab().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_not_enable_bit_logic_operation() {
        let value = ez_eval("true == true", &mut EmptyNamespace).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_not_enable_bit_and() {
        let value = ez_eval("1010 & 0010", &mut EmptyNamespace).unwrap();
    }

    #[test]
    fn compile_test() {
        compile().unwrap();
    }
}

pub fn compile() -> Result<(), fasteval::Error> {
    let parser = Parser::new();
    let mut slab = Slab::new();
    let mut context = BTreeMap::new();

    let expr_str = "sin(deg/360 * 2*pi())";
    let compiled = parser
        .parse(expr_str, &mut slab.ps)?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);

    for deg in 0..360 {
        context.insert("deg".to_string(), deg as f64);// update context

        // When working with compiled constant expressions, you can use the
        // eval_compiled*!() macros to save a function call:
        let val = eval_compiled!(compiled, &slab, &mut context);
        eprintln!("sin({}°) = {}", deg, val);
    }

    Ok(())
}

fn slab() -> Result<(), fasteval::Error> {
    let parser = fasteval::Parser::new();
    let mut slab = fasteval::Slab::new();

    // See the `parse` documentation to understand why we use `from` like this:
    let old_expr_ref = parser.parse("x + 1", &mut slab.ps)?.from(&slab.ps);

    // Let's evaluate the expression a couple times with different 'x' values:
    let mut map : BTreeMap<String,f64> = BTreeMap::new();
    map.insert("x".to_string(), 1.0);
    let val = old_expr_ref.eval(&slab, &mut map)?;
    assert_eq!(val, 2.0);

    map.insert("x".to_string(), 2.5);
    let val = old_expr_ref.eval(&slab, &mut map)?;
    assert_eq!(val, 3.5);

    // Now, let's re-use the Slab for a new expression.
    // (This is much cheaper than allocating a new Slab.)
    // --> 최적화

    // The Slab gets cleared by 'parse()', so you must avoid using
    // the old_expr_ref after parsing the new expression.
    // --> old_expr_ref 더이상 사용하면 안됨

    // One simple way to avoid this problem is to shadow the old variable:
    // --> shadowing을 사용하여 예전 expr 사용을 회피할 수 있음
    let expr_ref = parser.parse("x * 10", &mut slab.ps)?.from(&slab.ps);

    let val = expr_ref.eval(&slab, &mut map)?;
    assert_eq!(val, 25.0);

    Ok(())
}