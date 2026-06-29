test_inference! {
    name = array_literals,
    cases = {
        "<?php [];" => "array{}",
        "<?php [1, 2, 3];" => "list{0: int(1), 1: int(2), 2: int(3)}",
        "<?php ['a' => 1, 'b' => 2];" => "array{'a': int(1), 'b': int(2)}",
        "<?php [5 => 'x', 'y'];" => "array{5: string('x'), 6: string('y')}",
        "<?php ['1' => 'a', '01' => 'b'];" => "array{1: string('a'), '01': string('b')}",
        "<?php [true => 'a', null => 'b'];" => "array{1: string('a'), '': string('b')}",
        "<?php ['a' => 1, 'a' => 2];" => "array{'a': int(2)}",
    }
}
