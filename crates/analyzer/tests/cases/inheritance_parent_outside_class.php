<?php

declare(strict_types=1);

function inh_no_parent(): void {
    /** @mago-expect analysis:parent-outside-class-scope */
    parent::something();
}
