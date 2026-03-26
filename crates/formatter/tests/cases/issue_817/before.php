<?php

if (
    $self->gotMoneys()
    && (
        // Easter
        $today->isEaster() ||
        // XMAS
        $today->isXmas()
    )
) {
}
