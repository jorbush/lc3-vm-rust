
#[derive[Debug, Clone, Copy]]
enum ConditionFlag {
    Pos = 1 << 0, /* P */
    Zro = 1 << 1, /* Z */
    Neg = 1 << 2, /* N */
}
