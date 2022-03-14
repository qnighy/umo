const f = (x) => x + 10n;
assert_eq(f(f(3n)), 23n);
const compose = (f, g) => (x) => f(g(x));
assert_eq(compose((x) => x + 10n, (x) => x + 20n)(3n), 33n);
