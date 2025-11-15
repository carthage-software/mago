<?php

declare(strict_types=1);

namespace App\Parent {
    class Test
    {
        /**
         * @param  string  $_mask  e.g. '<presenter>/<action>/<id \d{1,3}>'
         * @param  array  $_metadata  default values or metadata
         * @return static
         */
        public function add(string $_mask, array $_metadata = [], int $_oneWay = 0): static
        {
            return $this;
        }
    }
}

namespace App\Child {
    use Override;

    class Test extends \App\Parent\Test
    {
        #[Override]
        public function add(
            string $_mask,
            array|string|\Closure $_metadata = [],
            int|bool $_oneWay = 0,
        ): static {
            echo 'hehe';
            return $this;
        }
    }
}

namespace App {

    use App\Child\Test;

    final class C
    {
        public static function c(): void
        {
            $test = new Test;

            // Should emit no errors - child's signature accepts string for $_metadata
            $test->add('asdf', 'asdf');
        }
    }
}
