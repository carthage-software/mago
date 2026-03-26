<?php

declare(strict_types=1);

final readonly class ThisOrThat
{
    public function __construct(
        public bool $a,
        public bool $b,
    ) {}
}

function is_this_or_that(ThisOrThat $thisOrThat): bool
{
    if ($thisOrThat->a || $thisOrThat->b) {
        print 'Either A or B is true';

        if ($thisOrThat->a) {
            print 'Returning A true';
        }

        if ($thisOrThat->b) {
            print 'Returning B true';
        }

        return true;
    }

    return false;
}

function is_this_or_that2(ThisOrThat $thisOrThat): bool
{
    if ($thisOrThat->a || $thisOrThat->b) {
        print 'Either A or B is true';

        if ($thisOrThat->a) {
            print 'Returning A true';
        } else if ($thisOrThat->b) { // @mago-expect analysis:redundant-condition
            print 'Returning B true';
        }

        return true;
    }

    return false;
}
