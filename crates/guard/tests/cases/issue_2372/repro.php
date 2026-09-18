<?php

declare(strict_types=1);

namespace App\Foo {
    class Thing {}
}

namespace App\Bar {
    class Thing {}
}

namespace App\Baz {
    class Thing {}
}

namespace App\LayerConsumer {
    new \App\Foo\Thing();
    new \App\Bar\Thing();
    new \App\Baz\Thing();
}

namespace App\PermitConsumer {
    new \App\Foo\Thing();
    new \App\Bar\Thing();
    new \App\Baz\Thing();
}

namespace App\TypedPermitConsumer {
    new \App\Foo\Thing();
    new \App\Bar\Thing();
    new \App\Baz\Thing();
}

namespace App\RestrictedConsumer {
    new \App\Foo\Thing();
    new \App\Bar\Thing();
    new \App\Baz\Thing();
}
