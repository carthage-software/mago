<?php

declare(strict_types=1);

class X {
    public function foo(): void
    {
        $this->derp(
            'zzzzzzzzzzzzzz',
            $this->bar(['zzzzzzzzzzzzzzzzzzzzz' => 'zzzzzzzzzzzzzz'], 'zzzzzzzzzzzzzzzzzzz')
                ->baz(
                    'zzzzzzzzzzzzzzzzzzzz',
                    'zzzzzzzzzzzzzzzzzzzz',
                    'zzzzzzzzzzzzzzzzzzzz',
                    'zzzzzzzzzzzzzzzzzzzz',
                    'zzzzzzzzzzzzzzzzzzzz',
                )
        );
    }
}
