test_inference! {
    name = unary_prefix,
    cases = {
        "<?php -5;" => "int(-5)",
        "<?php -1.5;" => "float(-1.5)",
        "<?php -'5';" => "int(-5)",
        "<?php -true;" => "int(-1)",
        "<?php +'5';" => "int(5)",
        "<?php +'1.5';" => "float(1.5)",
        "<?php +true;" => "int(1)",
        "<?php !true;" => "false",
        "<?php !0;" => "true",
        "<?php !'a';" => "false",
        "<?php ~5;" => "int(-6)",
        "<?php ~1.9;" => "int(-2)",
        "<?php 2 ** -1;" => "float(0.5)",
    }
}

test_inference! {
    name = casts,
    cases = {
        "<?php (int) 1.9;" => "int(1)",
        "<?php (int) '123abc';" => "int(123)",
        "<?php (int) true;" => "int(1)",
        "<?php (int) null;" => "int(0)",
        "<?php (float) 3;" => "float(3)",
        "<?php (bool) 0;" => "false",
        "<?php (bool) 'a';" => "true",
        "<?php (string) 1;" => "string('1')",
        "<?php (string) true;" => "string('1')",
        "<?php (string) null;" => "string('')",
    }
}

test_inference! {
    name = array_casts,
    cases = {
        "<?php (int) (float) [1];" => "int(1)",
        "<?php (int) [];" => "int(0)",
        "<?php (int) [1, 2];" => "int(1)",
        "<?php (float) [1];" => "float(1)",
        "<?php (float) [];" => "float(0)",
        "<?php (bool) [];" => "false",
        "<?php (bool) [1];" => "true",
        "<?php ![];" => "true",
        "<?php ![1];" => "false",
    }
}

test_inference! {
    name = increment_decrement,
    cases = {
        "<?php $i = 5; ++$i;" => "int(6)",
        "<?php $i = 5; --$i;" => "int(4)",
        "<?php $i = 5; $i++;" => "int(5)",
        "<?php $i = 5; $i++; $i;" => "int(6)",
        "<?php $i = 5; $i--; $i;" => "int(4)",
        "<?php $f = 1.5; ++$f;" => "float(2.5)",
        "<?php $n = null; ++$n;" => "int(1)",
        "<?php $n = null; --$n;" => "null",
        "<?php $s = '9'; ++$s;" => "int(10)",
    }
}

test_inference! {
    name = string_increment,
    cases = {
        "<?php $s = 'a'; ++$s;" => "string('b')",
        "<?php $s = 'Az'; ++$s;" => "string('Ba')",
        "<?php $s = 'Zz'; ++$s;" => "string('AAa')",
        "<?php $s = 'a9'; ++$s;" => "string('b0')",
        "<?php $s = 'abc'; --$s;" => "string('abc')",
    }
}

test_inference! {
    name = nested_casts,
    cases = {
        "<?php (int) (string) (int) (string) (1 + 2);" => "int(3)",
    }
}
