use auto_promoters::Int;
use num::{BigInt, Signed};

#[test]
fn test_int_machine_arithmetic() {
    let a = Int::from(100i64);
    let b = Int::from(-50i64);

    assert_eq!(a.clone() + b.clone(), Int::from(50i64));
    assert_eq!(a.clone() - b.clone(), Int::from(150i64));
    assert_eq!(a.clone() * b.clone(), Int::from(-5000i64));
    assert_eq!(a.clone() / b.clone(), Int::from(-2i64));
    assert_eq!(a.clone() % Int::from(30i64), Int::from(10i64));
    assert_eq!(-b.clone(), Int::from(50i64));
    assert_eq!(b.abs(), Int::from(50i64));
}

#[test]
fn test_int_auto_promotion_on_overflow() {
    let max = Int::from(i64::MAX);
    let one = Int::from(1i64);

    let promoted = max + one;
    match &promoted {
        Int::Promoted(big) => {
            assert_eq!(big, &(BigInt::from(i64::MAX) + 1i32));
        }
        _ => panic!("Expected auto-promotion to BigInt on i64 overflow!"),
    }

    let min = Int::from(i64::MIN);
    let neg_promoted = -min;
    match &neg_promoted {
        Int::Promoted(big) => {
            assert_eq!(big, &(-BigInt::from(i64::MIN)));
        }
        _ => panic!("Expected auto-promotion to BigInt on -i64::MIN!"),
    }
}

#[test]
fn test_int_demotion_back_to_machine() {
    let big = BigInt::from(i64::MAX) + 10i32;
    let promoted = Int::from(big);
    assert!(promoted.is_promoted());

    let demoted = promoted - Int::from(20i64);
    match demoted {
        Int::Machine(val) => {
            assert_eq!(val, i64::MAX - 10);
        }
        _ => panic!("Expected demotion back to Machine word!"),
    }
}
