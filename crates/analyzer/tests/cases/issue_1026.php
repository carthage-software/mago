<?php

declare(strict_types=1);

class User {}

class Agent {}

class Wrapper
{
    public function __construct(
        public readonly string $type,
    ) {}
}

/**
 * @param array<User|Agent> $items
 */
function process(array $items): void
{
    foreach ($items as $item) {
        if ($item instanceof User) {
            $wrapper = new Wrapper('user');
        } else {
            if ($item instanceof Agent) { // @mago-expect analysis:redundant-condition
                $wrapper = new Wrapper('agent');
            }
        }

        echo $wrapper->type;
    }

    foreach ($items as $item) {
        if ($item instanceof User) {
            $wrapper = new Wrapper('user');
        } else if ($item instanceof Agent) { // @mago-expect analysis:redundant-condition
            $wrapper = new Wrapper('agent');
        }

        echo $wrapper->type;
    }
}

process([new User(), new Agent()]);
