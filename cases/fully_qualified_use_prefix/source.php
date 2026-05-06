<?php

namespace Foo {
    class Bar {}

    class Baz {}
}

namespace Qux {
    use \Foo\Bar;
    use \Foo\Baz;

    function qux(Baz $baz, Bar $bar): void
    {
        \Quux\quux($bar, $baz);
    }
}

namespace Quux {
    use \Foo\Bar;
    use \Foo\Baz;

    function quux(Bar $bar, Baz $baz): void
    {
        \Qux\qux($baz, $bar);
    }
}
