use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_format_expressions() {
    let code = indoc! {r#"
        <?php

        declare(strict_types = 1);

        namespace InvalidBinaryOperatorWithMixed;

        /**
         * @template T
         * @param T $a
         */
        function genericMixed(mixed $a): void
        {
           	var_dump($a . 'a');
           	$b = 'a';
           	$b .= $a;
           	$bool = rand() > 0;
           	var_dump($a ** 2);
           	var_dump($a * 2);
           	var_dump($a / 2);
           	var_dump($a % 2);
           	var_dump($a + 2);
           	var_dump($a - 2);
           	var_dump($a << 2);
           	var_dump($a >> 2);
           	var_dump($a & 2);
           	var_dump($a | 2);
           	$c = 5;
           	$c += $a;
           	$c = 5;
           	$c -= $a;
           	$c = 5;
           	$c *= $a;
           	$c = 5;
           	$c **= $a;
           	$c = 5;
           	$c /= $a;
           	$c = 5;
           	$c %= $a;
           	$c = 5;
           	$c &= $a;
           	$c = 5;
           	$c |= $a;
           	$c = 5;
           	$c ^= $a;
           	$c = 5;
           	$c <<= $a;
           	$c = 5;
           	$c >>= $a;
        }
    "#};

    let expected = indoc! {r#"
        <?php

        declare(strict_types=1);

        namespace InvalidBinaryOperatorWithMixed;

        /**
         * @template T
         * @param T $a
         */
        function genericMixed(mixed $a): void
        {
            var_dump($a . 'a');
            $b = 'a';
            $b .= $a;
            $bool = rand() > 0;
            var_dump($a ** 2);
            var_dump($a * 2);
            var_dump($a / 2);
            var_dump($a % 2);
            var_dump($a + 2);
            var_dump($a - 2);
            var_dump($a << 2);
            var_dump($a >> 2);
            var_dump($a & 2);
            var_dump($a | 2);
            $c = 5;
            $c += $a;
            $c = 5;
            $c -= $a;
            $c = 5;
            $c *= $a;
            $c = 5;
            $c **= $a;
            $c = 5;
            $c /= $a;
            $c = 5;
            $c %= $a;
            $c = 5;
            $c &= $a;
            $c = 5;
            $c |= $a;
            $c = 5;
            $c ^= $a;
            $c = 5;
            $c <<= $a;
            $c = 5;
            $c >>= $a;
        }
    "#};

    test_format(code, expected, FormatSettings::default())
}
